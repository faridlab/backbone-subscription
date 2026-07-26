-- Down: drop subscription.subscription_plan_lines table
DROP TABLE IF EXISTS subscription.subscription_plan_lines CASCADE;
DROP FUNCTION IF EXISTS subscription.subscription_plan_lines_audit_timestamp() CASCADE;
