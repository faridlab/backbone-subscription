-- Revert the ADR-0014 strict fence re-statement for subscription module.
-- The fence predates this migration (ADR-0008-era), so the honest reverse is to
-- re-state the same live policy, not to disarm the tables: a down that disabled RLS
-- would leave company data unfenced — a posture this module never had.

-- Re-state the pre-existing fence for subscription.subscription_billing_runs (identical policy; see header).
DROP POLICY IF EXISTS subscription_billing_runs_company_isolation ON subscription.subscription_billing_runs;
CREATE POLICY subscription_billing_runs_company_isolation ON subscription.subscription_billing_runs
    FOR ALL
    USING      (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);

-- Re-state the pre-existing fence for subscription.subscription_plans (identical policy; see header).
DROP POLICY IF EXISTS subscription_plans_company_isolation ON subscription.subscription_plans;
CREATE POLICY subscription_plans_company_isolation ON subscription.subscription_plans
    FOR ALL
    USING      (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);

-- Re-state the pre-existing fence for subscription.subscriptions (identical policy; see header).
DROP POLICY IF EXISTS subscriptions_company_isolation ON subscription.subscriptions;
CREATE POLICY subscriptions_company_isolation ON subscription.subscriptions
    FOR ALL
    USING      (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);

