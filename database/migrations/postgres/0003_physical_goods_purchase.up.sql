ALTER TABLE commerce_order
    ADD COLUMN IF NOT EXISTS merchant_organization_id TEXT,
    ADD COLUMN IF NOT EXISTS shop_id TEXT,
    ADD COLUMN IF NOT EXISTS shipping_address_snapshot_json TEXT,
    ADD COLUMN IF NOT EXISTS shop_snapshot_json TEXT;

ALTER TABLE commerce_order_item
    ADD COLUMN IF NOT EXISTS product_id TEXT,
    ADD COLUMN IF NOT EXISTS shop_id TEXT;

CREATE TABLE IF NOT EXISTS commerce_checkout_session (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    checkout_session_no TEXT NOT NULL,
    owner_user_id TEXT NOT NULL,
    source_type TEXT NOT NULL,
    status TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    promotion_snapshot_json TEXT NOT NULL DEFAULT '[]',
    request_hash TEXT NOT NULL,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    shop_id TEXT,
    merchant_organization_id TEXT,
    shop_snapshot_json TEXT,
    shipping_address_snapshot_json TEXT,
    expires_at TEXT,
    submitted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_checkout_session_idempotency
    ON commerce_checkout_session(tenant_id, owner_user_id, idempotency_key);

CREATE TABLE IF NOT EXISTS commerce_checkout_line (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    checkout_session_id TEXT NOT NULL,
    product_id TEXT,
    shop_id TEXT,
    sku_id TEXT NOT NULL,
    sku_snapshot_json TEXT NOT NULL,
    selected_options_hash TEXT,
    quantity BIGINT NOT NULL,
    purchase_type TEXT NOT NULL,
    fulfillment_type TEXT NOT NULL,
    price_amount_snapshot TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    selected SMALLINT NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_checkout_line_session
    ON commerce_checkout_line(tenant_id, checkout_session_id, created_at, id);

CREATE TABLE IF NOT EXISTS commerce_checkout_quote (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    checkout_session_id TEXT NOT NULL,
    quote_no TEXT NOT NULL,
    original_amount TEXT NOT NULL,
    discount_amount TEXT NOT NULL,
    payable_amount TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    quote_status TEXT NOT NULL,
    expires_at TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_checkout_quote_session
    ON commerce_checkout_quote(tenant_id, checkout_session_id, created_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS commerce_fulfillment_order (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    fulfillment_no TEXT NOT NULL,
    order_id TEXT NOT NULL,
    fulfillment_type TEXT NOT NULL,
    status TEXT NOT NULL,
    warehouse_id TEXT,
    address_snapshot_id TEXT,
    provider_code TEXT,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_fulfillment_order_type
    ON commerce_fulfillment_order(tenant_id, order_id, fulfillment_type);
