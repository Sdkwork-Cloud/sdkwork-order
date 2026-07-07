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

  const cancelPost =
    appSpec.paths["/app/v3/api/orders/{orderId}/cancel"]?.post;
  assert.ok(cancelPost?.["x-sdkwork-idempotent"]);
  assert.ok(
    cancelPost?.parameters?.some(
      (entry) =>
        entry?.$ref === "#/components/parameters/WriteCommandRequestHash" ||
        entry?.name === "Sdkwork-Request-Hash",
    ),
  );

  for (const spec of [appSpec, backendSpec]) {
    for (const [path, methods] of Object.entries(spec.paths ?? {})) {
      for (const [method, operation] of Object.entries(methods ?? {})) {
        if (!operation?.["x-sdkwork-idempotent"]) {
          continue;
        }
        const params = operation.parameters ?? [];
        const hasIdempotency = params.some(
          (entry) =>
            entry?.$ref === "#/components/parameters/WriteCommandIdempotencyKey" ||
            entry?.name === "Idempotency-Key",
        );
        const hasRequestHash = params.some(
          (entry) =>
            entry?.$ref === "#/components/parameters/WriteCommandRequestHash" ||
            entry?.name === "Sdkwork-Request-Hash",
        );
        const hasBodyFingerprint = params.some(
          (entry) =>
            entry?.$ref === "#/components/parameters/WriteCommandIdempotencyFingerprint" ||
            entry?.name === "X-Idempotency-Fingerprint",
        );
        assert.ok(
          hasIdempotency && hasRequestHash && hasBodyFingerprint,
          `${method.toUpperCase()} ${path} (${operation.operationId}) must declare write-command headers`,
        );
      }
    }
  }
});

test("sdk openapi inputs stay aligned with api authorities", () => {
  const pairs = [
    [
      "apis/app-api/order/order-app-api.openapi.json",
      "sdks/sdkwork-order-app-sdk/openapi/sdkwork-order-app-api.openapi.json",
    ],
    [
      "apis/backend-api/order/order-backend-api.openapi.json",
      "sdks/sdkwork-order-backend-sdk/openapi/sdkwork-order-backend-api.openapi.json",
    ],
  ];

  for (const [authorityPath, sdkPath] of pairs) {
    const authority = fs.readFileSync(path.join(repoRoot, authorityPath), "utf8");
    const sdkCopy = fs.readFileSync(path.join(repoRoot, sdkPath), "utf8");
    assert.equal(
      sdkCopy,
      authority,
      `${sdkPath} must match ${authorityPath}; run pnpm sync:openapi`,
    );
  }
});

test("order route manifests are exported from gateway assembly", () => {
  const assemblyLib = fs.readFileSync(
    path.join(repoRoot, "crates/sdkwork-order-gateway-assembly/src/lib.rs"),
    "utf8",
  );
  assert.match(assemblyLib, /order_contract_fallback_config/);
});

test("standalone gateway CORS allows SDK write-command headers", () => {
  const mainRs = fs.readFileSync(
    path.join(repoRoot, "crates/sdkwork-order-standalone-gateway/src/main.rs"),
    "utf8",
  );

  for (const header of [
    "idempotency-key",
    "sdkwork-request-hash",
    "x-idempotency-fingerprint",
  ]) {
    assert.match(mainRs, new RegExp(`from_static\\("${header}"\\)`));
  }
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
