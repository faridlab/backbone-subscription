-- Down: drop the declared rule constraints for subscription module
ALTER TABLE IF EXISTS subscription.subscription_plan_lines DROP CONSTRAINT IF EXISTS ck_subscription_plan_lines_unit_price_no_5e313b69__non_negative;
ALTER TABLE IF EXISTS subscription.subscription_plan_lines DROP CONSTRAINT IF EXISTS ck_subscription_plan_lines_quantity_non_negative__non_negative;
ALTER TABLE IF EXISTS subscription.subscription_plan_lines DROP CONSTRAINT IF EXISTS ck_subscription_plan_lines_account_id_required__required;
ALTER TABLE IF EXISTS subscription.subscription_plan_lines DROP CONSTRAINT IF EXISTS ck_subscription_plan_lines_item_id_required__required;
ALTER TABLE IF EXISTS subscription.subscription_plan_lines DROP CONSTRAINT IF EXISTS ck_subscription_plan_lines_plan_id_required__required;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_trial_days_non_negative__non_negative;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_price_non_negative__non_negative;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_receivable_account_id_required__required;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_currency_max_length__max_length;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_billing_day_range__range;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_name_max_length__max_length;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_name_required__required;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_plan_code_max_length__max_length;
ALTER TABLE IF EXISTS subscription.subscription_plans DROP CONSTRAINT IF EXISTS ck_subscription_plans_plan_code_required__required;
DO $$
BEGIN
    IF to_regclass('subscription.uq_subscription_plans_plan_code__unique') IS NOT NULL AND to_regclass('subscription.idx_subscription_plans_plan_code') IS NULL THEN
        ALTER INDEX subscription.uq_subscription_plans_plan_code__unique RENAME TO idx_subscription_plans_plan_code;
    END IF;
END $$;

ALTER TABLE IF EXISTS subscription.subscription_billing_runs DROP CONSTRAINT IF EXISTS ck_subscription_billing_runs_idempotency_k_c7954ffd__max_length;
ALTER TABLE IF EXISTS subscription.subscription_billing_runs DROP CONSTRAINT IF EXISTS ck_subscription_billing_runs_idempotency_key_required__required;
ALTER TABLE IF EXISTS subscription.subscription_billing_runs DROP CONSTRAINT IF EXISTS ck_subscription_billing_runs_grand_total_577fe4c7__non_negative;
ALTER TABLE IF EXISTS subscription.subscription_billing_runs DROP CONSTRAINT IF EXISTS ck_subscription_billing_runs_subscription_id_required__required;
DO $$
BEGIN
    IF to_regclass('subscription.uq_subscription_billing_runs_idempotency_key__unique') IS NOT NULL AND to_regclass('subscription.idx_subscription_billing_runs_idempotency_key') IS NULL THEN
        ALTER INDEX subscription.uq_subscription_billing_runs_idempotency_key__unique RENAME TO idx_subscription_billing_runs_idempotency_key;
    END IF;
END $$;

DO $$
BEGIN
    IF to_regclass('subscription.uq_subscription_billing_runs_period_start__unique') IS NOT NULL AND to_regclass('subscription.idx_subscription_billing_runs_subscription_id_period_start') IS NULL THEN
        ALTER INDEX subscription.uq_subscription_billing_runs_period_start__unique RENAME TO idx_subscription_billing_runs_subscription_id_period_start;
    END IF;
END $$;

ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS ck_subscriptions_current_period_start__invalid_period_dates;
ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS ck_subscriptions_currency_max_length__max_length;
ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS ck_subscriptions_plan_id_required__required;
ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS ck_subscriptions_customer_id_required__required;
ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS ck_subscriptions_subscription_number_max_length__max_length;
ALTER TABLE IF EXISTS subscription.subscriptions DROP CONSTRAINT IF EXISTS ck_subscriptions_subscription_number_required__required;
DO $$
BEGIN
    IF to_regclass('subscription.uq_subscriptions_subscription_number__unique') IS NOT NULL AND to_regclass('subscription.idx_subscriptions_subscription_number') IS NULL THEN
        ALTER INDEX subscription.uq_subscriptions_subscription_number__unique RENAME TO idx_subscriptions_subscription_number;
    END IF;
END $$;

