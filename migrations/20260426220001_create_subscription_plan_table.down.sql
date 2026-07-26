-- Down: drop subscription.subscription_plans table
DROP TABLE IF EXISTS subscription.subscription_plans CASCADE;
DROP FUNCTION IF EXISTS subscription.subscription_plans_audit_timestamp() CASCADE;
