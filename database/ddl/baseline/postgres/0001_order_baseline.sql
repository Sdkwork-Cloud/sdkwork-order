-- sdkwork:baseline
-- module: order
-- order-owned core and account-value tables

CREATE TABLE IF NOT EXISTS commerce_order (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    owner_user_id TEXT NOT NULL,
    order_no TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending_payment',
    subject TEXT NOT NULL,
    currency_code TEXT NOT NULL DEFAULT 'CNY',
    payment_status TEXT,
    fulfillment_status TEXT,
    refund_status TEXT,
    request_no TEXT,
    idempotency_key TEXT,
    created_at TEXT NOT NULL,
    paid_at TEXT,
    cancelled_at TEXT,
    expired_at TEXT,
    updated_at TEXT NOT NULL
);

ALTER TABLE commerce_order ADD COLUMN IF NOT EXISTS payment_status TEXT;
ALTER TABLE commerce_order ADD COLUMN IF NOT EXISTS fulfillment_status TEXT;
ALTER TABLE commerce_order ADD COLUMN IF NOT EXISTS refund_status TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS uk_order_owner_idempotency
    ON commerce_order(tenant_id, COALESCE(organization_id, '0'), owner_user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_order_owner_list
    ON commerce_order(tenant_id, organization_id, owner_user_id, created_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS commerce_order_item (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    order_id TEXT NOT NULL,
    sku_id TEXT,
    sku_snapshot_json TEXT,
    title TEXT,
    quantity BIGINT NOT NULL DEFAULT 1,
    unit_price_amount TEXT,
    discount_amount TEXT,
    tax_amount TEXT,
    total_amount TEXT,
    fulfillment_status TEXT,
    refund_status TEXT,
    created_at TEXT NOT NULL
);

ALTER TABLE commerce_order_item ADD COLUMN IF NOT EXISTS sku_snapshot_json TEXT;
ALTER TABLE commerce_order_item ADD COLUMN IF NOT EXISTS title TEXT;
ALTER TABLE commerce_order_item ADD COLUMN IF NOT EXISTS discount_amount TEXT;
ALTER TABLE commerce_order_item ADD COLUMN IF NOT EXISTS tax_amount TEXT;
ALTER TABLE commerce_order_item ADD COLUMN IF NOT EXISTS fulfillment_status TEXT;
ALTER TABLE commerce_order_item ADD COLUMN IF NOT EXISTS refund_status TEXT;

CREATE INDEX IF NOT EXISTS idx_order_item_order
    ON commerce_order_item(tenant_id, order_id, created_at, id);

CREATE TABLE IF NOT EXISTS commerce_order_amount_breakdown (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    order_id TEXT NOT NULL,
    order_item_id TEXT,
    allocation_type TEXT NOT NULL DEFAULT 'order_total',
    original_amount TEXT NOT NULL DEFAULT '0',
    discount_amount TEXT NOT NULL DEFAULT '0',
    payable_amount TEXT NOT NULL DEFAULT '0',
    currency_code TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

ALTER TABLE commerce_order_amount_breakdown ADD COLUMN IF NOT EXISTS order_item_id TEXT;
ALTER TABLE commerce_order_amount_breakdown ADD COLUMN IF NOT EXISTS original_amount TEXT NOT NULL DEFAULT '0';

CREATE INDEX IF NOT EXISTS idx_order_amount_breakdown_order
    ON commerce_order_amount_breakdown(tenant_id, order_id, allocation_type, id);

CREATE TABLE IF NOT EXISTS commerce_recharge_package (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    external_id BIGINT NOT NULL,
    package_no TEXT NOT NULL,
    sku_id TEXT NOT NULL,
    name TEXT NOT NULL,
    price_amount TEXT NOT NULL,
    currency_code TEXT NOT NULL DEFAULT 'CNY',
    bonus_points BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    valid_from TEXT,
    valid_to TEXT,
    sort_weight BIGINT NOT NULL DEFAULT 0,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_recharge_package_no
    ON commerce_recharge_package(tenant_id, COALESCE(organization_id, '0'), package_no);

CREATE UNIQUE INDEX IF NOT EXISTS uk_recharge_package_idempotency
    ON commerce_recharge_package(tenant_id, COALESCE(organization_id, '0'), idempotency_key);

CREATE INDEX IF NOT EXISTS idx_recharge_package_list
    ON commerce_recharge_package(tenant_id, organization_id, status, sort_weight, id);

CREATE INDEX IF NOT EXISTS idx_recharge_package_sku
    ON commerce_recharge_package(tenant_id, sku_id);

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
