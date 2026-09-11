-- Hand-authored (user-owned). Not regenerated.
--
-- Strip every company-fence artifact from the subscription tables (ADR-0029): the module is
-- tenant-agnostic; org scoping is installed by the COMPOSING service's tenancy decorator,
-- never by the module. Dropped here, per table: the company-leading index, the
-- <table>_company_isolation RLS policy, and the company_id column itself.
--
-- Ordering guard (the decorator must run FIRST on any database with data): the module
-- never moves tenancy data. A table is safe to strip when EITHER
--   a) it carries org_unit_id with no NULLs — the decorator backfilled it from company_id —
--      or b) it is empty (a fresh database: the earlier chain files created it empty).
-- Otherwise the strip RAISEs, naming the decorator step, rather than dropping a column
-- that still holds the only tenancy key. The file is re-runnable (every drop is IF EXISTS
-- and the tracker has no checksums), so a failed run retries cleanly after the decorator
-- lands.
--
-- RLS enable/force flags are deliberately NOT touched: the decorator owns those now.
-- subscription_plan_lines has no company_id (it is parent-scoped through
-- subscription_plans), so there is nothing to strip there. The domain uniques (plan_code,
-- subscription_number, subscription+period, idempotency_key) are already tenant-free and
-- stay; the cadence's due-date read path (idx_subscriptions_due) is domain, not posture.

DO $$
DECLARE
    t text;
    has_org boolean;
    org_nulls bigint;
    total bigint;
    offenders text := '';
BEGIN
    FOREACH t IN ARRAY ARRAY['subscription_plans', 'subscriptions', 'subscription_billing_runs']
    LOOP
        IF to_regclass(format('subscription.%I', t)) IS NULL THEN
            CONTINUE; -- chain not fully applied on this database; nothing to strip
        END IF;

        SELECT EXISTS (
                   SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'subscription' AND table_name = t AND column_name = 'org_unit_id'
               )
        INTO has_org;

        EXECUTE format('SELECT count(*) FROM subscription.%I', t) INTO total;

        IF has_org THEN
            EXECUTE format(
                'SELECT count(*) FROM subscription.%I WHERE org_unit_id IS NULL', t)
            INTO org_nulls;
        ELSE
            org_nulls := total; -- no org column: every row's only tenancy key is company_id
        END IF;

        IF has_org AND org_nulls = 0 THEN
            CONTINUE; -- decorator backfilled: safe
        END IF;
        IF total = 0 THEN
            CONTINUE; -- empty table (fresh database): safe
        END IF;
        offenders := offenders || format(' subscription.%s (%s rows, %s rows not covered by org_unit_id);', t, total, org_nulls);
    END LOOP;

    IF offenders <> '' THEN
        RAISE EXCEPTION 'refusing to strip company_id — these tables are not yet covered by the tenancy decorator:%. Apply the composing service''s tenancy decorator (it backfills org_unit_id from company_id) and re-run; it is the only step that moves tenancy data.', offenders;
    END IF;
END $$;

-- ── subscription_plans ─────────────────────────────────────────────────────────
DROP INDEX IF EXISTS subscription.idx_subscription_plans_company_id_status;
DROP POLICY IF EXISTS subscription_plans_company_isolation ON subscription.subscription_plans;
ALTER TABLE subscription.subscription_plans DROP COLUMN IF EXISTS company_id;

-- ── subscriptions ──────────────────────────────────────────────────────────────
DROP INDEX IF EXISTS subscription.idx_subscriptions_company_id_customer_id_status;
DROP POLICY IF EXISTS subscriptions_company_isolation ON subscription.subscriptions;
ALTER TABLE subscription.subscriptions DROP COLUMN IF EXISTS company_id;

-- ── subscription_billing_runs ──────────────────────────────────────────────────
DROP INDEX IF EXISTS subscription.idx_subscription_billing_runs_company_id_status;
DROP POLICY IF EXISTS subscription_billing_runs_company_isolation ON subscription.subscription_billing_runs;
ALTER TABLE subscription.subscription_billing_runs DROP COLUMN IF EXISTS company_id;
