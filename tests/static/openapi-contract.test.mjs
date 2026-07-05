import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("order openapi authorities declare v3 list and command envelopes", () => {
  const appOpenApiPath = path.join(
    repoRoot,
    "apis/app-api/order/order-app-api.openapi.json",
  );
  const backendOpenApiPath = path.join(
    repoRoot,
    "apis/backend-api/order/order-backend-api.openapi.json",
  );

  const appSpec = JSON.parse(fs.readFileSync(appOpenApiPath, "utf8"));
  const backendSpec = JSON.parse(fs.readFileSync(backendOpenApiPath, "utf8"));

  assert.ok(appSpec.components?.schemas?.SdkWorkListResponse);
  assert.ok(appSpec.components?.schemas?.SdkWorkCommandResponse);
  assert.ok(appSpec.paths["/app/v3/api/orders/{orderId}/payments"]?.get);
  assert.ok(appSpec.paths["/app/v3/api/recharges/packages"]?.get?.parameters?.length > 0);
  assert.ok(
    appSpec.paths["/app/v3/api/orders/payments/webhooks/{providerCode}"]?.post,
    "PSP webhook route must be on order app-api",
  );

  assert.ok(backendSpec.components?.schemas?.SdkWorkCommandResponse);
  assert.ok(backendSpec.paths["/backend/v3/api/orders"]?.get);
  assert.ok(
    backendSpec.paths["/backend/v3/api/orders/{orderId}/payment_confirmations"]?.post,
    "manual payment confirmation must be on order backend-api",
  );
});

test("order route manifests are exported from gateway assembly", () => {
  const assemblyLib = fs.readFileSync(
    path.join(repoRoot, "crates/sdkwork-order-gateway-assembly/src/lib.rs"),
    "utf8",
  );
  assert.match(assemblyLib, /order_contract_fallback_config/);
});

test("payment webhook spec declares order ownership", () => {
  const spec = JSON.parse(
    fs.readFileSync(
      path.join(repoRoot, "specs/commerce-payment-webhook.spec.json"),
      "utf8",
    ),
  );
  assert.match(
    spec.ownedRoutes.app[0],
    /\/app\/v3\/api\/orders\/payments\/webhooks/,
  );
  assert.ok(spec.forbidden.some((entry) => entry.includes("sdkwork-payment")));
});
