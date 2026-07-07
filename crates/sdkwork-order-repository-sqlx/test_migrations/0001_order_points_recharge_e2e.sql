-- sdkwork:migration
-- id: 0001_order_points_recharge_e2e
-- engine: sqlite
-- module: order
-- purpose: Minimal commerce order + payment tables for points-recharge store E2E tests
-- reversible: true
-- transactional: true

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

CREATE TABLE IF NOT EXISTS commerce_order_item (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    sku_id TEXT,
    sku_snapshot_json TEXT,
    title TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price_amount TEXT,
    total_amount TEXT,
    fulfillment_status TEXT,
    refund_status TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_payment_intent (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    owner_user_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    status TEXT NOT NULL,
    amount TEXT,
    currency_code TEXT,
    payment_method TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_payment_attempt (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    owner_user_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    status TEXT NOT NULL,
    amount TEXT,
    currency_code TEXT,
    payment_method TEXT,
    paid_at TEXT,
    callback_payload TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_idempotency_key (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    scope TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    response_json TEXT,
    locked_until TEXT,
    expires_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_commerce_idempotency_key_scope
    ON commerce_idempotency_key(tenant_id, scope, idempotency_key);

-- After-sales lifecycle tables (refund / return / exchange)
CREATE TABLE IF NOT EXISTS commerce_after_sales_request (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    after_sales_no TEXT NOT NULL,
    order_id TEXT NOT NULL,
    owner_user_id TEXT NOT NULL,
    after_sales_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'submitted',
    refund_status TEXT NOT NULL DEFAULT 'none',
    return_status TEXT NOT NULL DEFAULT 'none',
    exchange_status TEXT NOT NULL DEFAULT 'none',
    reason_code TEXT NOT NULL,
    description TEXT,
    requested_amount TEXT NOT NULL,
    approved_amount TEXT NOT NULL DEFAULT '0.00',
    currency_code TEXT NOT NULL,
    requested_by_type TEXT NOT NULL DEFAULT 'buyer',
    requested_by TEXT,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_after_sales_request_tenant_owner
    ON commerce_after_sales_request(tenant_id, owner_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_after_sales_request_idempotency
    ON commerce_after_sales_request(tenant_id, order_id, idempotency_key);

CREATE TABLE IF NOT EXISTS commerce_after_sales_request_item (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    after_sales_id TEXT NOT NULL,
    order_item_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    requested_amount TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (after_sales_id) REFERENCES commerce_after_sales_request(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_after_sales_request_item_request
    ON commerce_after_sales_request_item(after_sales_id);

CREATE TABLE IF NOT EXISTS commerce_after_sales_event (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    after_sales_id TEXT NOT NULL,
    event_no TEXT NOT NULL,
    event_type TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT NOT NULL,
    actor_type TEXT NOT NULL DEFAULT 'buyer',
    actor_id TEXT,
    request_id TEXT,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (after_sales_id) REFERENCES commerce_after_sales_request(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_after_sales_event_request
    ON commerce_after_sales_event(tenant_id, after_sales_id, created_at ASC);

CREATE TABLE IF NOT EXISTS commerce_after_sales_return_shipment (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    after_sales_id TEXT NOT NULL,
    return_shipment_no TEXT NOT NULL,
    carrier_code TEXT,
    tracking_no TEXT,
    status TEXT NOT NULL DEFAULT 'submitted',
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (after_sales_id) REFERENCES commerce_after_sales_request(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_after_sales_return_shipment_idempotency
    ON commerce_after_sales_return_shipment(tenant_id, after_sales_id, idempotency_key);

CREATE TABLE IF NOT EXISTS commerce_order_event (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    event_no TEXT NOT NULL,
    order_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT NOT NULL,
    actor_type TEXT,
    actor_id TEXT,
    reason_code TEXT,
    message TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}',
    request_id TEXT,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_order_cancellation (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    status TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    reason_message TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_order_amount_breakdown (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    order_id TEXT NOT NULL,
    order_item_id TEXT,
    allocation_type TEXT NOT NULL DEFAULT 'order_total',
    original_amount TEXT,
    discount_amount TEXT,
    payable_amount TEXT,
    currency_code TEXT,
    created_at TEXT NOT NULL
);

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

CREATE TABLE IF NOT EXISTS commerce_shipment (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    shipment_no TEXT NOT NULL,
    fulfillment_id TEXT NOT NULL,
    carrier_code TEXT NOT NULL,
    tracking_no TEXT,
    status TEXT NOT NULL,
    shipped_at TEXT,
    delivered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_commerce_shipment_tenant_created
    ON commerce_shipment(tenant_id, created_at DESC);

CREATE TABLE IF NOT EXISTS commerce_shipment_package (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    shipment_id TEXT NOT NULL,
    package_no TEXT NOT NULL,
    package_type TEXT NOT NULL,
    tracking_no TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_commerce_shipment_package_shipment
    ON commerce_shipment_package(tenant_id, shipment_id, created_at ASC);

CREATE TABLE IF NOT EXISTS commerce_shipment_tracking_event (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    shipment_id TEXT NOT NULL,
    tracking_event_no TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_status TEXT,
    event_time TEXT NOT NULL,
    location_text TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_product_spu (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    spu_no TEXT NOT NULL,
    name TEXT,
    title TEXT,
    sales_status TEXT NOT NULL DEFAULT 'active',
    status TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_product_sku (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    spu_id TEXT NOT NULL,
    sku_no TEXT NOT NULL,
    name TEXT,
    title TEXT,
    price_amount TEXT,
    currency_code TEXT,
    sales_status TEXT NOT NULL DEFAULT 'active',
    status TEXT,
    spec_json TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS membership_package_group (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT,
    organization_id TEXT,
    external_id INTEGER,
    group_no TEXT,
    name TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    sort_weight INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS membership_package (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT,
    organization_id TEXT,
    external_id INTEGER NOT NULL,
    package_no TEXT,
    package_group_id TEXT NOT NULL,
    sku_id TEXT,
    name TEXT NOT NULL,
    price_amount TEXT NOT NULL,
    currency_code TEXT NOT NULL DEFAULT 'CNY',
    duration_days INTEGER NOT NULL,
    sort_weight INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commerce_payment_method (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    method_key TEXT NOT NULL,
    provider TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    sort_weight INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO commerce_product_spu (
    id, tenant_id, organization_id, spu_no, name, title, sales_status, status, created_at, updated_at
) VALUES (
    'seed-product-membership', '100001', '0', 'membership-catalog', 'Membership Catalog', 'Membership Catalog', 'active', 'active', datetime('now'), datetime('now')
);

INSERT OR IGNORE INTO commerce_product_sku (
    id, tenant_id, organization_id, spu_id, sku_no, name, title, price_amount, currency_code, sales_status, status, created_at, updated_at
) VALUES (
    'sku-basic-monthly', '100001', '0', 'seed-product-membership', 'basic-monthly', 'Basic Monthly', 'Basic Monthly', '68.00', 'CNY', 'active', 'active', datetime('now'), datetime('now')
);

INSERT OR IGNORE INTO membership_package_group (
    id, tenant_id, organization_id, external_id, group_no, name, status, sort_weight, created_at, updated_at
) VALUES (
    'package-group-monthly', '100001', '0', 2, 'monthly', '连续包月', 'active', 1, datetime('now'), datetime('now')
);

INSERT OR IGNORE INTO membership_package (
    id, tenant_id, organization_id, external_id, package_no, package_group_id, sku_id, name, price_amount, currency_code, duration_days, sort_weight, status, created_at, updated_at
) VALUES (
    'package-basic-monthly', '100001', '0', 201, 'basic-monthly', 'package-group-monthly', 'sku-basic-monthly', '基础会员·月卡', '68.00', 'CNY', 30, 1, 'active', datetime('now'), datetime('now')
);

INSERT OR IGNORE INTO commerce_payment_method (
    id, tenant_id, organization_id, method_key, provider, status, sort_weight, created_at, updated_at
) VALUES (
    'pm-wechat', '100001', '0', 'wechat_pay', 'wechat_pay', 'active', 1, datetime('now'), datetime('now')
);
