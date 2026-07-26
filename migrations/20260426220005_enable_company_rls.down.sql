-- Down: remove the company RLS fence for subscription module

-- Reverse the company RLS fence for subscription.subscriptions
DROP POLICY IF EXISTS subscriptions_company_isolation ON subscription.subscriptions;
ALTER TABLE subscription.subscriptions NO FORCE ROW LEVEL SECURITY;
ALTER TABLE subscription.subscriptions DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for subscription.subscription_billing_runs
DROP POLICY IF EXISTS subscription_billing_runs_company_isolation ON subscription.subscription_billing_runs;
ALTER TABLE subscription.subscription_billing_runs NO FORCE ROW LEVEL SECURITY;
ALTER TABLE subscription.subscription_billing_runs DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for subscription.subscription_plans
DROP POLICY IF EXISTS subscription_plans_company_isolation ON subscription.subscription_plans;
ALTER TABLE subscription.subscription_plans NO FORCE ROW LEVEL SECURITY;
ALTER TABLE subscription.subscription_plans DISABLE ROW LEVEL SECURITY;

