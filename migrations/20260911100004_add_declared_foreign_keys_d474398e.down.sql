-- Down: drop the declared foreign keys for subscription module
ALTER TABLE IF EXISTS subscription.subscription_plan_lines DROP CONSTRAINT IF EXISTS fk_subscription_plan_lines_plan_id;
ALTER TABLE IF EXISTS subscription.subscription_billing_runs DROP CONSTRAINT IF EXISTS fk_subscription_billing_runs_subscription_id;
ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS fk_subscriptions_plan_id;
