-- Platform points recharge catalog. Amounts are stored as major-unit decimals.
-- The SKU ids are stable references to the shared commerce merchandise catalog.
-- Retire the legacy demo catalog that was previously bootstrapped outside Order ownership.
UPDATE commerce_recharge_package
SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
WHERE tenant_id = '100001'
  AND (organization_id = '0' OR organization_id IS NULL)
  AND id LIKE 'bootstrap-admin-recharge-package-%';

INSERT INTO commerce_recharge_package (
    id, tenant_id, organization_id, external_id, package_no, sku_id, name,
    price_amount, currency_code, bonus_points, status, valid_from, valid_to,
    sort_weight, request_no, idempotency_key, created_at, updated_at
) VALUES
    ('recharge-500', '100001', '0', 500, 'points-500', 'recharge-sku-500', '500 compute points', '50.00', 'CNY', 0, 'active', NULL, NULL, 10, 'seed-recharge-500', 'seed-recharge-500', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('recharge-750', '100001', '0', 750, 'points-750', 'recharge-sku-750', '750 compute points', '75.00', 'CNY', 0, 'active', NULL, NULL, 20, 'seed-recharge-750', 'seed-recharge-750', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('recharge-1500', '100001', '0', 1500, 'points-1500', 'recharge-sku-1500', '1500 compute points', '150.00', 'CNY', 0, 'active', NULL, NULL, 30, 'seed-recharge-1500', 'seed-recharge-1500', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('recharge-2250', '100001', '0', 2250, 'points-2250', 'recharge-sku-2250', '2250 compute points', '223.00', 'CNY', 20, 'active', NULL, NULL, 40, 'seed-recharge-2250', 'seed-recharge-2250', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('recharge-4500', '100001', '0', 4500, 'points-4500', 'recharge-sku-4500', '4500 compute points', '450.00', 'CNY', 0, 'active', NULL, NULL, 50, 'seed-recharge-4500', 'seed-recharge-4500', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('recharge-9000', '100001', '0', 9000, 'points-9000', 'recharge-sku-9000', '9000 compute points', '899.00', 'CNY', 10, 'active', NULL, NULL, 60, 'seed-recharge-9000', 'seed-recharge-9000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    tenant_id = EXCLUDED.tenant_id,
    organization_id = EXCLUDED.organization_id,
    external_id = EXCLUDED.external_id,
    package_no = EXCLUDED.package_no,
    sku_id = EXCLUDED.sku_id,
    name = EXCLUDED.name,
    price_amount = EXCLUDED.price_amount,
    currency_code = EXCLUDED.currency_code,
    bonus_points = EXCLUDED.bonus_points,
    status = EXCLUDED.status,
    valid_from = EXCLUDED.valid_from,
    valid_to = EXCLUDED.valid_to,
    sort_weight = EXCLUDED.sort_weight,
    request_no = EXCLUDED.request_no,
    idempotency_key = EXCLUDED.idempotency_key,
    updated_at = EXCLUDED.updated_at;
