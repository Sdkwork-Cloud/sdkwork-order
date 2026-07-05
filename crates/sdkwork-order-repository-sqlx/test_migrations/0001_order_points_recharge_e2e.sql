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
    allocation_type TEXT NOT NULL,
    original_amount TEXT,
    discount_amount TEXT,
    payable_amount TEXT,
    currency_code TEXT,
    created_at TEXT NOT NULL
);
