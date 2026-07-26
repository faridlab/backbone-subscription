//! Subscription domain events (hand-authored, user-owned) — the public seam contract.
//!
//! The cadence stages a `SubscriptionInvoiceDue` into the durable outbox; a composition-layer
//! relay turns it into `billing.create_sales_invoice` + `post_sales_invoice`. The shipped library
//! has zero normal Cargo edge into billing — the event is the only handoff (mirrors the
//! payment→billing inbound seam).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One line of the invoice blueprint carried on the due event (mirrors billing's `NewInvoiceLine`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DueLine {
    pub item_id: Uuid,
    pub account_id: Uuid,
    pub description: Option<String>,
    pub quantity: Decimal,
    pub unit_price: Decimal,
}

/// Emitted when a subscription's period is due. The composition ACL maps this to a
/// `NewSalesInvoice` and calls billing to create + post it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionInvoiceDue {
    pub subscription_id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub plan_id: Uuid,
    pub posting_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub currency: Option<String>,
    pub receivable_account_id: Uuid,
    /// The plan's blueprint lines (item + qty + price) → billing's invoice lines.
    pub lines: Vec<DueLine>,
    /// The amount the generated invoice must total (carried on the event, ADR-002 §2 lesson).
    pub grand_total: Decimal,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
}

/// The subscription domain-event union (for the in-proc sink + outbox routing).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SubscriptionEvent {
    SubscriptionInvoiceDue(SubscriptionInvoiceDue),
}

/// Fire-and-forget sink for the in-proc path (a real adapter wires a bus; tests record).
/// Mirrors billing's `BillingEventSink`.
pub trait SubscriptionEventSink: Send + Sync {
    fn publish(&self, event: SubscriptionEvent);
}

/// Default sink — emits a structured tracing event.
pub struct LoggingSink;
impl SubscriptionEventSink for LoggingSink {
    fn publish(&self, event: SubscriptionEvent) {
        tracing::info!(target: "subscription.events", ?event, "subscription domain event");
    }
}
