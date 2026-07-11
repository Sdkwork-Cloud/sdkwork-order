-- sdkwork:baseline
-- module: order
-- account value order-owned tables; shared commerce core tables are reference-only in this repo

CREATE TABLE IF NOT EXISTS commerce_account_value_package (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    package_code TEXT NOT NULL,
    display_name TEXT NOT NULL,
    target_asset TEXT NOT NULL,
    grant_amount TEXT NOT NULL,
    bonus_amount TEXT NOT NULL DEFAULT '0',
    price_amount TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    sort_weight BIGINT NOT NULL DEFAULT 0,
    valid_from TEXT,
    valid_to TEXT,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    retired_at TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_account_value_package_code
    ON commerce_account_value_package(tenant_id, COALESCE(organization_id, '0'), package_code);

CREATE UNIQUE INDEX IF NOT EXISTS uk_account_value_package_idempotency
    ON commerce_account_value_package(tenant_id, COALESCE(organization_id, '0'), idempotency_key);

CREATE INDEX IF NOT EXISTS idx_account_value_package_list
    ON commerce_account_value_package(tenant_id, organization_id, target_asset, status, sort_weight, id);

CREATE TABLE IF NOT EXISTS commerce_token_bank_plan (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    plan_code TEXT NOT NULL,
    display_name TEXT NOT NULL,
    plan_period TEXT NOT NULL,
    grant_amount TEXT NOT NULL,
    bonus_amount TEXT NOT NULL DEFAULT '0',
    price_amount TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    renewal_policy TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    sort_weight BIGINT NOT NULL DEFAULT 0,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    retired_at TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_token_bank_plan_code
    ON commerce_token_bank_plan(tenant_id, COALESCE(organization_id, '0'), plan_code);

CREATE UNIQUE INDEX IF NOT EXISTS uk_token_bank_plan_idempotency
    ON commerce_token_bank_plan(tenant_id, COALESCE(organization_id, '0'), idempotency_key);

CREATE INDEX IF NOT EXISTS idx_token_bank_plan_list
    ON commerce_token_bank_plan(tenant_id, organization_id, status, sort_weight, plan_code);

CREATE TABLE IF NOT EXISTS commerce_order_refund_request (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    request_no TEXT NOT NULL,
    original_order_id TEXT NOT NULL,
    owner_user_id TEXT NOT NULL,
    target_asset TEXT NOT NULL,
    amount TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    provider_amount TEXT,
    provider_currency_code TEXT,
    status TEXT NOT NULL DEFAULT 'requested',
    reason_code TEXT,
    reason_detail TEXT,
    review_comment TEXT,
    provider_reference_id TEXT,
    account_effect_reference_id TEXT,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_order_refund_request_idempotency
    ON commerce_order_refund_request(tenant_id, COALESCE(organization_id, '0'), owner_user_id, idempotency_key);

CREATE INDEX IF NOT EXISTS idx_order_refund_request_owner
    ON commerce_order_refund_request(tenant_id, organization_id, owner_user_id, status, created_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS commerce_order_withdrawal_request (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    request_no TEXT NOT NULL,
    owner_user_id TEXT NOT NULL,
    target_asset TEXT NOT NULL,
    amount TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    provider_amount TEXT,
    provider_currency_code TEXT,
    status TEXT NOT NULL DEFAULT 'requested',
    payout_method TEXT,
    payout_account_ref TEXT,
    reason_code TEXT,
    review_comment TEXT,
    provider_reference_id TEXT,
    account_effect_reference_id TEXT,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_order_withdrawal_request_idempotency
    ON commerce_order_withdrawal_request(tenant_id, COALESCE(organization_id, '0'), owner_user_id, idempotency_key);

CREATE INDEX IF NOT EXISTS idx_order_withdrawal_request_owner
    ON commerce_order_withdrawal_request(tenant_id, organization_id, owner_user_id, status, created_at DESC, id DESC);
