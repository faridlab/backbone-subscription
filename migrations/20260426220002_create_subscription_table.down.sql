-- Down: drop subscription.subscriptions table
DROP TABLE IF EXISTS subscription.subscriptions CASCADE;
DROP FUNCTION IF EXISTS subscription.subscriptions_audit_timestamp() CASCADE;
