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
//! NOTE (v1): the cadence's read/advance SQL lives here rather than in a `_recurrence_repository.rs`.
//! It is parameterized and read-heavy; extract it to a repository if it grows beyond this engine.
//! Uses runtime-checked sqlx (string queries), like backbone-billing's repositories.

use backbone_orm::company_scope;
use chrono::{Months, NaiveDate};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use super::subscription_events::{
    DueLine, LoggingSink, SubscriptionEvent, SubscriptionEventSink, SubscriptionInvoiceDue,
};

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
        company_scope::bind_company_on(&mut tx, d.company_id).await?;

        // Gate 1: the period advance. WHERE next_billing_date = $old so a concurrent tick that
        // already advanced this subscription misses (rows_affected == 0) → idempotent.
        let advanced: u64 = sqlx::query(
            r#"UPDATE subscription.subscriptions
                 SET current_period_start = $2, current_period_end = $3, next_billing_date = $4
               WHERE id = $1 AND next_billing_date = $5 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(d.id).bind(period_start).bind(period_end).bind(new_next).bind(d.next_billing_date)
        .execute(&mut *tx).await?.rows_affected();
        if advanced == 0 {
            tx.rollback().await?;
            return Ok(false);
        }

        // Gate 2: exactly one billing run per (subscription, period). ON CONFLICT DO NOTHING +
        // RETURNING — if a run already exists (re-tick), no new run, no event.
        let run_row = sqlx::query(
            r#"INSERT INTO subscription.subscription_billing_runs
                 (id, subscription_id, company_id, period_start, period_end, due_date,
                  grand_total, status, idempotency_key)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending'::billing_run_status, $8)
               ON CONFLICT (subscription_id, period_start) WHERE (metadata->>'deleted_at') IS NULL
               DO NOTHING RETURNING id"#,
        )
        .bind(Uuid::new_v4()).bind(d.id).bind(d.company_id)
        .bind(period_start).bind(period_end).bind(period_start)
        .bind(d.grand_total).bind(format!("{}:{}", d.id, period_start))
        .fetch_optional(&mut *tx).await?;

        if let Some(row) = run_row {
            let _run_id: Uuid = row.get("id");
            // Winner: stage the seam event in the same tx (the fence).
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
            subscription_id: d.id, company_id: d.company_id, customer_id: d.customer_id,
            branch_id: d.branch_id, plan_id: d.plan_id, posting_date: period_start,
            due_date: Some(period_start), currency: Some(d.currency.clone()),
            receivable_account_id: d.receivable_account_id, lines: d.lines.clone(),
            grand_total: d.grand_total, period_start, period_end,
        }
    }

    /// Stage a serialized seam event into the outbox on the shared transition tx (mirrors
    /// backbone-billing::stage_outbox_event). Company scope is already bound on `conn`.
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
        let rec = backbone_outbox::OutboxRecord::new(
            event_type, aggregate_type, aggregate_id.to_string(), payload, chrono::Utc::now(),
        );
        backbone_outbox::outbox::stage(&mut *conn, schema, &rec)
            .await
            .map_err(|e| SubscriptionError::Db(sqlx::Error::Protocol(e.to_string())))?;
        Ok(())
    }

    /// Read every active, due subscription joined with its plan + plan lines (the engine's unit of
    /// work). v1: plan lines fetched per-row — acceptable for a bounded cadence batch.
    async fn fetch_due(&self, today: NaiveDate) -> Result<Vec<DueSubscription>, SubscriptionError> {
        #[derive(FromRow)]
        struct PlanRow {
            id: Uuid, company_id: Uuid, customer_id: Uuid, branch_id: Option<Uuid>, plan_id: Uuid,
            next_billing_date: NaiveDate, currency: String, billing_cycle: String,
            receivable_account_id: Uuid,
        }
        let plans: Vec<PlanRow> = sqlx::query_as(
            r#"SELECT s.id, s.company_id, s.customer_id, s.branch_id, s.plan_id,
                      s.next_billing_date, s.currency,
                      p.billing_cycle::text AS billing_cycle, p.receivable_account_id
               FROM subscription.subscriptions s
               JOIN subscription.subscription_plans p ON s.plan_id = p.id
               WHERE s.status = 'active' AND s.next_billing_date <= $1
                 AND (s.metadata->>'deleted_at') IS NULL"#,
        )
        .bind(today)
        .fetch_all(&self.db_pool).await?;

        let mut out = Vec::with_capacity(plans.len());
        for p in plans {
            #[derive(FromRow)]
            struct LineRow {
                item_id: Uuid, account_id: Uuid, description: Option<String>,
                quantity: Decimal, unit_price: Decimal,
            }
            let line_rows: Vec<LineRow> = sqlx::query_as(
                r#"SELECT item_id, account_id, description, quantity, unit_price
                   FROM subscription.subscription_plan_lines
                   WHERE plan_id = $1 AND (metadata->>'deleted_at') IS NULL"#,
            )
            .bind(p.plan_id)
            .fetch_all(&self.db_pool).await?;
            let lines: Vec<DueLine> = line_rows.into_iter()
                .map(|l| DueLine { item_id: l.item_id, account_id: l.account_id, description: l.description,
                                   quantity: l.quantity, unit_price: l.unit_price })
                .collect();
            let grand_total = money(lines.iter().map(|l| l.quantity * l.unit_price).sum());
            out.push(DueSubscription {
                id: p.id, company_id: p.company_id, customer_id: p.customer_id, branch_id: p.branch_id,
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
    id: Uuid, company_id: Uuid, customer_id: Uuid, branch_id: Option<Uuid>, plan_id: Uuid,
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
