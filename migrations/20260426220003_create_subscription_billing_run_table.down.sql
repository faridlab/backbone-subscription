-- Down: drop subscription.subscription_billing_runs table
DROP TABLE IF EXISTS subscription.subscription_billing_runs CASCADE;
DROP FUNCTION IF EXISTS subscription.subscription_billing_runs_audit_timestamp() CASCADE;
