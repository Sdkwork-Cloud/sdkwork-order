-- sdkwork:migration
-- version: 0002
-- engine: postgres
-- module: order
-- description: Persist membership request fingerprints and reusable purchase intents.
-- reversible: false
-- transactional: true
-- lock: table
-- contract_version: 1.0.0

BEGIN;

ALTER TABLE commerce_order
    ADD COLUMN IF NOT EXISTS request_fingerprint TEXT,
    ADD COLUMN IF NOT EXISTS purchase_intent_key TEXT,
    ADD COLUMN IF NOT EXISTS membership_action TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS uk_membership_order_active_purchase_intent
    ON commerce_order (
        tenant_id,
        COALESCE(organization_id, '0'),
        owner_user_id,
        subject,
        purchase_intent_key
    )
    WHERE purchase_intent_key IS NOT NULL
      AND status IN ('draft', 'pending', 'pending_payment', 'unpaid', 'wait_pay', 'created');

COMMIT;
