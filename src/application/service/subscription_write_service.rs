//! The subscription write service (hand-authored, user-owned) — owns the cadence unit of work.
//!
//! `process_due(today)` finds every active subscription whose `next_billing_date` has arrived,
//! advances its period + next_billing_date (per the plan's billing_cycle), records a
//! `SubscriptionBillingRun`, and stages a `SubscriptionInvoiceDue` into the durable outbox — all in
//! ONE transaction per subscription, so a crash after the advance can never lose the event (mirrors
//! backbone-billing/backbone-payment's atomic transition+stage). A composition-layer relay drains
//! the outbox → billing.create_sales_invoice + post_sales_invoice → writes the invoice_id back onto
//! the billing run.
//!
//! NOTE: the cadence's read/advance SQL lives in the subscription repositories
//! (`SubscriptionRepository::find_due_plans`/`advance_period`, `SubscriptionBillingRunRepository`,
//! `SubscriptionPlanLineRepository`) per the module's 4-layer rule — this service orchestrates only.
//! `find_due_plans` is a fleet-wide sweep across every org unit: the module is tenant-agnostic
//! (ADR-0029) and the composing service's tenancy decorator scopes its reads — the composition
//! layer must drive `process_due` from a context the decorator does not fence (jobs lane), or the
//! scoped read returns 0 rows and nobody is billed.

use backbone_orm::org_scope;
use chrono::{Months, NaiveDate};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::persistence::{
    NewBillingRunRow, SubscriptionBillingRunRepository, SubscriptionPlanLineRepository,
    SubscriptionRepository,
};

use super::subscription_events::{
    DueLine, LoggingSink, SubscriptionEvent, SubscriptionEventSink, SubscriptionInvoiceDue,
};

// --- legacy tenancy twin ------------------------------------------------------

/// The legacy tenancy twin echo (ADR-0029): outbound wire shapes that still carry a `company_id`
/// (the durable-outbox record, the seam event's envelope) get the ambient org scope's legacy
/// company id when the composing service bound one; nil otherwise. Nothing in this module keys a
/// statement on it, and an undecorated deployment is unfenced by design.
fn legacy_company_echo() -> Uuid {
    org_scope::current_org_scope()
        .and_then(|s| s.legacy_company_id())
        .unwrap_or(Uuid::nil())
}

/// Re-bind the caller's ambient org scope onto a transaction this service opened itself — the
/// scope is task-local and a fresh pool transaction carries none of it. With no ambient scope
/// (standalone deployment, jobs) the transaction stays plain: the module is tenant-agnostic and
/// the composed decorator owns isolation.
async fn relay_ambient_scope(tx: &mut sqlx::PgConnection) -> Result<(), SubscriptionError> {
    if let Some(scope) = org_scope::current_org_scope() {
        org_scope::bind_org_scope_on(tx, &scope)
            .await
            .map_err(SubscriptionError::Db)?;
    }
    Ok(())
}

// --- errors -----------------------------------------------------------------

#[derive(Debug)]
pub enum SubscriptionError {
    Db(sqlx::Error),
}
impl std::fmt::Display for SubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { SubscriptionError::Db(e) => write!(f, "db: {e}") }
    }
}
impl std::error::Error for SubscriptionError {}
impl From<sqlx::Error> for SubscriptionError {
    fn from(e: sqlx::Error) -> Self { SubscriptionError::Db(e) }
}

// --- service ----------------------------------------------------------------

#[derive(Clone)]
pub struct SubscriptionWriteService {
    db_pool: PgPool,
    sink: Arc<dyn SubscriptionEventSink>,
    /// When set, `process_due` stages `SubscriptionInvoiceDue` into `<schema>.outbox_events`
    /// atomically with the period advance (go-live durable bus; mirrors backbone-payment).
    outbox_schema: Option<String>,
}

impl SubscriptionWriteService {
    pub fn new(db_pool: PgPool) -> Self {
        Self::with_sink(db_pool, Arc::new(LoggingSink))
    }
    pub fn with_sink(db_pool: PgPool, sink: Arc<dyn SubscriptionEventSink>) -> Self {
        Self { db_pool, sink, outbox_schema: None }
    }
    pub fn with_outbox_schema(mut self, schema: impl Into<String>) -> Self {
        self.outbox_schema = Some(schema.into());
        self
    }

    /// The cadence tick. Returns the number of subscriptions billed this run.
    ///
    /// For each active subscription with `next_billing_date <= today`: advance the period +
    /// next_billing_date by one `billing_cycle`, insert a `SubscriptionBillingRun(pending)`, and
    /// stage `SubscriptionInvoiceDue` into the outbox — all in one tx. Idempotent: a re-tick of an
    /// already-advanced period is a no-op (the advance UPDATE misses + the billing_run unique
    /// constraint fires), so at-least-once cron delivery cannot double-bill.
    pub async fn process_due(&self, today: NaiveDate) -> Result<usize, SubscriptionError> {
        let due = self.fetch_due(today).await?;
        let mut billed = 0usize;
        for d in &due {
            if self.bill_one(d, today).await? {
                billed += 1;
            }
        }
        Ok(billed)
    }

    /// One subscription's billing unit of work. Returns true if THIS call performed the billing
    /// (false = a concurrent/re-run tick already did it; idempotent no-op).
    async fn bill_one(&self, d: &DueSubscription, _today: NaiveDate) -> Result<bool, SubscriptionError> {
        let Some(new_next) = advance(d.next_billing_date, d.billing_cycle.as_str()) else { return Ok(false) };
        let period_start = d.next_billing_date;
        let period_end = new_next.pred_opt().unwrap_or(new_next);

        let mut tx = self.db_pool.begin().await?;
        // Relay the caller's ambient org scope onto this transaction (ADR-0029): the composing
        // service's decorator set it task-locally; the fresh transaction carries none of it.
        relay_ambient_scope(&mut tx).await?;

        let subscriptions = SubscriptionRepository::new(self.db_pool.clone());
        let billing_runs = SubscriptionBillingRunRepository::new(self.db_pool.clone());

        // Gate 1: the period advance. WHERE next_billing_date = $old so a concurrent tick that
        // already advanced this subscription misses (rows_affected == 0) → idempotent.
        let advanced = subscriptions
            .advance_period(&mut *tx, d.id, period_start, period_end, new_next, d.next_billing_date)
            .await?;
        if advanced == 0 {
            tx.rollback().await?;
            return Ok(false);
        }

        // Gate 2: exactly one billing run per (subscription, period). ON CONFLICT DO NOTHING +
        // RETURNING — if a run already exists (re-tick), no new run, no event.
        let run_id = billing_runs
            .insert_pending_run_on_conflict_nothing(&mut *tx, &NewBillingRunRow {
                id: Uuid::new_v4(),
                subscription_id: d.id,
                period_start,
                period_end,
                due_date: period_start,
                grand_total: d.grand_total,
                idempotency_key: format!("{}:{}", d.id, period_start),
            })
            .await?;

        if run_id.is_some() {
            // Winner: stage the seam event in the same tx (the crash-safe fence).
            if let Some(schema) = self.outbox_schema.as_deref() {
                let event = self.due_event(d, period_start, period_end);
                self.stage_outbox_event(&mut *tx, schema, "SubscriptionInvoiceDue",
                    "Subscription", d.id, &event).await?;
            }
            tx.commit().await?;
            // In-proc sink fires after commit (best-effort; the durable path is the outbox).
            self.sink.publish(SubscriptionEvent::SubscriptionInvoiceDue(self.due_event(d, period_start, period_end)));
            Ok(true)
        } else {
            // Billing run already existed for this period (re-tick) but the advance won the UPDATE —
            // partial; roll back so we don't double-advance without a billing run.
            tx.rollback().await?;
            Ok(false)
        }
    }

    fn due_event(&self, d: &DueSubscription, period_start: NaiveDate, period_end: NaiveDate) -> SubscriptionInvoiceDue {
        SubscriptionInvoiceDue {
            subscription_id: d.id, company_id: legacy_company_echo(), customer_id: d.customer_id,
            branch_id: d.branch_id, plan_id: d.plan_id, posting_date: period_start,
            due_date: Some(period_start), currency: Some(d.currency.clone()),
            receivable_account_id: d.receivable_account_id, lines: d.lines.clone(),
            grand_total: d.grand_total, period_start, period_end,
        }
    }

    /// Stage a serialized seam event into the outbox on the shared transition tx (mirrors
    /// backbone-billing::stage_outbox_event). The ambient org scope is already relayed onto
    /// `conn`, so this just executes on it.
    async fn stage_outbox_event<E: serde::Serialize>(
        &self,
        conn: &mut sqlx::PgConnection,
        schema: &str,
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event: &E,
    ) -> Result<(), SubscriptionError> {
        let payload = serde_json::to_value(event)
            .map_err(|e| SubscriptionError::Db(sqlx::Error::Protocol(format!("outbox serialize: {e}"))))?;
        // OutboxRecord::new requires the owning tenant (ADR-0011 — the outbox_events table is fenced
        // by company_id). The module is tenant-agnostic, so the record carries the ambient scope's
        // legacy company echo; under a composed tenancy decorator the relay's fence reads it.
        let rec = backbone_outbox::OutboxRecord::new(
            event_type, aggregate_type, aggregate_id.to_string(), legacy_company_echo(), payload, chrono::Utc::now(),
        );
        backbone_outbox::outbox::stage(&mut *conn, schema, &rec)
            .await
            .map_err(|e| SubscriptionError::Db(sqlx::Error::Protocol(e.to_string())))?;
        Ok(())
    }

    /// Read every active, due subscription joined with its plan + plan lines (the engine's unit of
    /// work). v1: plan lines fetched per-row — acceptable for a bounded cadence batch.
    async fn fetch_due(&self, today: NaiveDate) -> Result<Vec<DueSubscription>, SubscriptionError> {
        let subscriptions = SubscriptionRepository::new(self.db_pool.clone());
        let plan_lines = SubscriptionPlanLineRepository::new(self.db_pool.clone());

        let plans = subscriptions.find_due_plans(&self.db_pool, today).await?;

        let mut out = Vec::with_capacity(plans.len());
        for p in plans {
            let line_rows = plan_lines.find_blueprint_by_plan(&self.db_pool, p.plan_id).await?;
            let lines: Vec<DueLine> = line_rows.into_iter()
                .map(|l| DueLine { item_id: l.item_id, account_id: l.account_id, description: l.description,
                                   quantity: l.quantity, unit_price: l.unit_price })
                .collect();
            let grand_total = money(lines.iter().map(|l| l.quantity * l.unit_price).sum());
            out.push(DueSubscription {
                id: p.id, customer_id: p.customer_id, branch_id: p.branch_id,
                plan_id: p.plan_id, next_billing_date: p.next_billing_date, currency: p.currency,
                billing_cycle: p.billing_cycle, receivable_account_id: p.receivable_account_id,
                lines, grand_total,
            });
        }
        Ok(out)
    }
}

// one due subscription + its plan's invoice blueprint (the engine's unit of work)
struct DueSubscription {
    id: Uuid, customer_id: Uuid, branch_id: Option<Uuid>, plan_id: Uuid,
    next_billing_date: NaiveDate, currency: String, billing_cycle: String,
    receivable_account_id: Uuid, lines: Vec<DueLine>, grand_total: Decimal,
}

/// 2dp half-away-from-zero (matches billing's money rounding).
fn money(v: Decimal) -> Decimal {
    use rust_decimal::RoundingStrategy;
    v.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Advance a billing date by one cycle. Monthly/quarterly/yearly use month arithmetic so the
/// billing day stays stable across month lengths.
fn advance(date: NaiveDate, cycle: &str) -> Option<NaiveDate> {
    match cycle {
        "monthly" => date.checked_add_months(Months::new(1)),
        "quarterly" => date.checked_add_months(Months::new(3)),
        "yearly" => date.checked_add_months(Months::new(12)),
        "weekly" => date.checked_add_days(chrono::Days::new(7)),
        _ => date.checked_add_days(chrono::Days::new(1)), // daily + fallback
    }
}
