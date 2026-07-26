-- Down: drop enum types for subscription module
DROP TYPE IF EXISTS subscription_plan_status CASCADE;
DROP TYPE IF EXISTS billing_cycle CASCADE;
DROP TYPE IF EXISTS billing_run_status CASCADE;
DROP TYPE IF EXISTS subscription_status CASCADE;
