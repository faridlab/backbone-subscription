//! SSEAM-1 — the subscription→billing seam, end-to-end:
//! cadence ticks → stages `SubscriptionInvoiceDue` into the outbox → a relay consumer creates +
//! posts a real Sales Invoice in backbone-billing → the `BillingRun` is invoiced.
//!
//! Zero normal Cargo edges into billing (dev-dep only for this test). Requires DATABASE_URL
//! (:5433/backbone_subscription with subscription + billing schemas co-migrated).

use std::sync::{Arc, Mutex};

use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use backbone_billing::application::service::billing_events::{BillingEvent, BillingEventSink};
use backbone_billing::application::service::billing_gl::{
    AccountingPostEnvelope, GlPostAck, GlPostRejected, GlPostSink,
};
use backbone_billing::application::service::billing_write_service::{
    BillingWriteService, NewInvoiceLine, NewSalesInvoice,
};
use backbone_subscription::application::service::subscription_events::SubscriptionInvoiceDue;
use backbone_subscription::application::service::subscription_write_service::SubscriptionWriteService;

fn d(s: &str) -> Decimal { Decimal::from_str_exact(s).unwrap() }

/// A GL sink that always acks (no real accounting needed — proves the billing post path).
struct OkGl;
#[async_trait::async_trait]
impl GlPostSink for OkGl {
    async fn post(&self, _e: &AccountingPostEnvelope) -> Result<GlPostAck, GlPostRejected> {
        Ok(GlPostAck { post_id: Uuid::new_v4(), journal_id: Uuid::new_v4(), idempotent_reuse: false })
    }
}

/// A recording billing-event sink (captures SalesInvoicePosted for assertion).
#[derive(Default, Clone)]
struct Rec { events: Arc<Mutex<Vec<BillingEvent>>> }
impl BillingEventSink for Rec {
    fn publish(&self, e: BillingEvent) { self.events.lock().unwrap().push(e); }
}

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5433/backbone_subscription".to_string());
    PgPool::connect(&url).await.expect("connect DB")
}

/// Seed a monthly plan with one line + an active subscription that is due today.
async fn seed_due_subscription(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let (customer, item, revenue_acct, ar_acct) = (
        Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(),
    );
    let plan_id = Uuid::new_v4();
    let today = chrono::Utc::now().date_naive();
    let last_month = today.checked_sub_months(chrono::Months::new(1)).unwrap();

    // Plan: monthly, 100,000 IDR, one line.
    sqlx::query(
        r#"INSERT INTO subscription.subscription_plans
             (id, plan_code, name, billing_cycle, billing_day, currency,
              receivable_account_id, price, status)
           VALUES ($1, $2, 'Pro Monthly', 'monthly', 1, 'IDR', $3, 100000, 'active')"#,
    )
    .bind(plan_id).bind(format!("PRO-{}", &Uuid::new_v4().simple().to_string()[..8]))
    .bind(ar_acct)
    .execute(pool).await.unwrap();

    // Plan line: 1 × 100,000 to a revenue account.
    sqlx::query(
        r#"INSERT INTO subscription.subscription_plan_lines
             (id, plan_id, item_id, account_id, quantity, unit_price)
           VALUES ($1, $2, $3, $4, 1, 100000)"#,
    )
    .bind(Uuid::new_v4()).bind(plan_id).bind(item).bind(revenue_acct)
    .execute(pool).await.unwrap();

    // Subscription: active, next_billing_date = today (due now).
    let sub_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO subscription.subscriptions
             (id, subscription_number, customer_id, plan_id, status,
              started_at, current_period_start, current_period_end, next_billing_date, currency)
           VALUES ($1, $2, $3, $4, 'active', $5, $5, $6, $6, 'IDR')"#,
    )
    .bind(sub_id)
    .bind(format!("SUB-{}", &Uuid::new_v4().simple().to_string()[..8]))
    .bind(customer).bind(plan_id)
    .bind(last_month).bind(today)
    .execute(pool).await.unwrap();

    (sub_id, customer, plan_id)
}

#[tokio::test]
async fn cadence_stages_invoice_due_and_relay_creates_billing_invoice() {
    let pool = pool().await;
    backbone_outbox::outbox::migrate(&pool, "subscription").await.expect("migrate subscription outbox");
    backbone_outbox::outbox::migrate(&pool, "billing").await.expect("migrate billing outbox");

    let (sub_id, customer, _plan_id) = seed_due_subscription(&pool).await;
    let today = chrono::Utc::now().date_naive();

    // 1) The cadence ticks.
    let svc = SubscriptionWriteService::new(pool.clone()).with_outbox_schema("subscription");
    let billed = svc.process_due(today).await.unwrap();
    assert_eq!(billed, 1, "one due subscription was billed");

    // 2) The subscription's next_billing_date advanced (monthly → +1 month from today).
    let next: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT next_billing_date FROM subscription.subscriptions WHERE id = $1")
        .bind(sub_id).fetch_one(&pool).await.unwrap();
    assert_eq!(next, today.checked_add_months(chrono::Months::new(1)).unwrap(),
        "monthly advance");

    // 3) The outbox holds exactly one SubscriptionInvoiceDue for this subscription.
    let row = sqlx::query(
        r#"SELECT payload, aggregate_id::text FROM subscription.outbox_events
           WHERE event_type = 'SubscriptionInvoiceDue' AND aggregate_id = $1"#)
        .bind(sub_id.to_string()).fetch_one(&pool).await.unwrap();
    let payload: serde_json::Value = row.get("payload");
    let agg_id: Uuid = row.get::<String, _>("aggregate_id").parse().unwrap();
    assert_eq!(agg_id, sub_id);
    let due: SubscriptionInvoiceDue = serde_json::from_value(payload).expect("deserialize");
    assert_eq!(due.customer_id, customer);
    assert_eq!(due.grand_total, d("100000.00"));
    assert_eq!(due.lines.len(), 1);
    assert_eq!(due.lines[0].unit_price, d("100000"));

    // 4) The relay: turn the event into a real billing invoice.
    let billing = BillingWriteService::new(pool.clone());
    let gl = OkGl;
    let rec = Rec::default();
    let billing_rec = BillingWriteService::with_sink(pool.clone(), Arc::new(rec.clone()));

    let inv = billing_rec.create_sales_invoice(NewSalesInvoice {
        invoice_number: format!("INV-{}", &Uuid::new_v4().simple().to_string()[..8]),
        // The event's company_id is the legacy tenancy twin (ADR-0029) — passed through for
        // the pinned billing release's still-company-shaped input; billing keys no fence on
        // it under its own strip.
        company_id: due.company_id, branch_id: due.branch_id, customer_id: due.customer_id,
        source_so_id: None, posting_date: due.posting_date, due_date: due.due_date,
        payment_term_id: None,
        currency: due.currency.clone(), receivable_account_id: due.receivable_account_id,
        lines: due.lines.iter().map(|l| NewInvoiceLine {
            item_id: l.item_id, account_id: l.account_id, description: l.description.clone(),
            quantity: l.quantity, unit_price: l.unit_price, tax_template_id: None,
        }).collect(),
        tax_lines: vec![],
    }).await.unwrap();
    billing_rec.post_sales_invoice(inv, &gl).await.unwrap();

    // 5) The billing invoice is posted (the full A/R path).
    let posted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM billing.sales_invoices WHERE id = $1 AND posting_state = 'posted'")
        .bind(inv).fetch_one(&pool).await.unwrap();
    assert_eq!(posted_count, 1, "the relay created a real posted Sales Invoice in billing");

    // 6) The relay marks the BillingRun invoiced (the composition ACL's writeback).
    let invoiced: i64 = sqlx::query(
        r#"UPDATE subscription.subscription_billing_runs
             SET status = 'invoiced'::billing_run_status, invoice_id = $2
           WHERE subscription_id = $1 AND status = 'pending'::billing_run_status
           RETURNING id"#)
        .bind(sub_id).bind(inv).fetch_all(&pool).await.unwrap().len() as i64;
    assert_eq!(invoiced, 1, "the billing run is marked invoiced with the real invoice id");

    let _ = billing; // keep the non-rec service alive for the scope
}
