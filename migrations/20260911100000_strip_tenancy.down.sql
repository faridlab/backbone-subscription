-- Hand-authored (user-owned). Not regenerated.
--
-- Best-effort restore sketch for the tenancy strip (ADR-0029). This is a breaking module
-- release against dev-stage databases: the down re-adds the company_id column as nullable
-- with the company-leading indexes in their final pre-strip shapes, but restores NO data —
-- rows written after the strip (or after the decorator re-keyed them) carry org_unit_id
-- only. The composing service's tenancy decorator remains the live fence; the
-- <table>_company_isolation policies are NOT recreated here. Treat this down as a
-- schema-shape sketch for archaeology, not a usable rollback.

ALTER TABLE subscription.subscription_plans          ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE subscription.subscriptions               ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE subscription.subscription_billing_runs   ADD COLUMN IF NOT EXISTS company_id uuid;

-- ── subscription_plans ─────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_subscription_plans_company_id_status
    ON subscription.subscription_plans (company_id, status);

-- ── subscriptions ──────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_subscriptions_company_id_customer_id_status
    ON subscription.subscriptions (company_id, customer_id, status);

-- ── subscription_billing_runs ──────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_subscription_billing_runs_company_id_status
    ON subscription.subscription_billing_runs (company_id, status);
