# ADR-001: Subscription owns plans, recurrence, and the cadence; billing owns the invoice

**Status**: Accepted — Applied 2026-07-26
**Deciders**: Farid (owner), build session 2026-07-26
**Related**: `backbone-billing` ADR-001 (billing boundary), ADR-002 (the seam pattern)

## Context

`backbone-billing` is a spot-invoicing engine — it creates, posts, and reverses individual A/R
and A/P invoices. It explicitly defers recurring/subscription billing (billing ADR-001 Consequences).
Recurring revenue — monthly SaaS charges, annual licenses, period-end billing — had no home. This
ADR records the boundary for `backbone-subscription`, the module that fills that gap.

## Decision

1. **Subscription owns the "when + what," billing owns the "document."** Subscription owns the
   *plan* (recurrence rule + invoice blueprint), the *subscription* instance (who/what/status/period),
   and the *billing run* (each cycle's record + idempotency). It decides WHEN to bill (the cadence)
   and WHAT to bill (the plan's blueprint). Billing owns the invoice document itself — subscription
   never creates or posts an invoice directly.

2. **The seam is an event + ACL, zero normal Cargo edges.** The cadence stages a serialized
   `SubscriptionInvoiceDue` into the durable outbox (atomic with the period advance). A
   composition-layer relay deserializes it → `billing.create_sales_invoice` + `post_sales_invoice`
   → writes the `invoice_id` back onto the `SubscriptionBillingRun`. `backbone-billing` is a
   dev-dep only (`cargo tree -e normal -i backbone-billing` is empty in the shipped crate).

3. **The cadence is jobs + outbox.** A `backbone-jobs` cron ticks `process_due(today)`; the
   existing per-schema outbox relay delivers the event. Two layers: jobs owns *when*, outbox owns
   *delivery durability*.

4. **Idempotency is structural.** A re-tick of the same period is a no-op: the period-advance
   `UPDATE WHERE next_billing_date = $old` misses (rows_affected == 0) + the `SubscriptionBillingRun`
   unique constraint on `(subscription_id, period_start)` fires. At-least-once cron delivery
   cannot double-bill.

## Consequences

- Proven end-to-end by `tests/subscription_seam.rs` (SSEAM-1): due subscription → outbox event →
  relay → real posted Sales Invoice in billing → BillingRun `invoiced`.
- Subscription is independently composable: it needs only a Postgres pool + `backbone-outbox`.
- Deferred (v1 parking lot): usage-based/metered billing, proration, mid-cycle upgrade/downgrade,
  trial complexity, payment retry/dunning (backbone-payment), tax computation (backbone-tax),
  multi-currency (backbone-banking).
