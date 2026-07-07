# Production Operations

Status: active  
Updated: 2026-07-07

## Deployment Topology

`sdkwork-order` ships as a standalone Axum gateway (`sdkwork-order-standalone-gateway`) exposing:

| Surface | Prefix | Auth |
| --- | --- | --- |
| App API | `/app/v3/api/*` | IAM dual-token |
| Backend API | `/backend/v3/api/*` | IAM dual-token + org scope |
| Infra | `/healthz`, `/livez`, `/readyz`, `/metrics` | Public (metrics should be in-cluster only) |

Run multiple instances behind a load balancer. All instances share the same PostgreSQL database and external IAM/account/payment dependencies.

## Environment Variables

| Variable | Required | Notes |
| --- | --- | --- |
| `ORDER_API_BIND` | No | Default `0.0.0.0:18093` |
| `ORDER_CORS_ALLOW_ORIGINS` | Production | Comma-separated browser origins; unset denies CORS |
| `SDKWORK_ORDER_PLATFORM_CATALOG_TENANT_ID` | No | Platform recharge catalog tenant (default `100001`) |
| `SDKWORK_ACCESS_TOKEN` | Production | Service credential for account wallet credit and membership fulfillment HTTP adapters |
| `SDKWORK_ORDER_MEMBERSHIP_FULFILLMENT_ADAPTER` | No | `http` (default) or `noop` for membership order settlement |
| `SDKWORK_MEMBERSHIP_BACKEND_API_ORIGIN` | Production | Membership backend for membership-subject fulfillment (default `http://127.0.0.1:18096`) |
| `ORDER_READ_MODEL_LENIENT` | No | **Forbidden in production.** Set `1` only for local scaffolding without commerce DDL |
| `ORDER_PAYMENT_WEBHOOK_BASE_URL` | Production | Public base URL for PSP notify: `{base}/app/v3/api/orders/payments/webhooks/{providerCode}` |
| `RUST_LOG` | No | e.g. `info,order.bootstrap=info,order.runtime=info` |

## Payment Webhooks

PSP notify URLs **must** target the **order gateway**, not `sdkwork-payment`:

```text
POST {ORDER_PAYMENT_WEBHOOK_BASE_URL}/app/v3/api/orders/payments/webhooks/{providerCode}
```

The legacy payment path `POST /app/v3/api/payments/webhooks/{providerCode}` returns **410 Gone**.

Operator manual settlement replay:

```text
POST /backend/v3/api/orders/{orderId}/payment_confirmations
```

Requires IAM permission `commerce.orders.fulfill`.

Duplicate PSP deliveries with the same `provider_event_id` are idempotent at the payment ingest layer (`replayed: true`) and do **not** re-run settlement. If payment succeeded but order settlement did not complete, replay via `payment_confirmations`.

## Health And Observability

- **Liveness:** `GET /healthz`, `GET /livez`
- **Readiness:** `GET /readyz` (includes database `SELECT 1`)
- **Metrics:** `GET /metrics` (Prometheus text; scrape in-cluster only, do not expose on public ingress)
- **Tracing:** structured logs with targets `order.bootstrap`, `order.runtime`, `order.readiness`, `order.security`
- **Contract fallback:** manifest-declared routes without handlers return HTTP 501; unknown paths return HTTP 404 (merged app + backend manifests)

## High Availability

1. Run **N ≥ 2** gateway replicas with the same DB connection pool limits tuned per instance.
2. Use PostgreSQL with automated backups and point-in-time recovery.
3. Configure `ORDER_CORS_ALLOW_ORIGINS` explicitly per environment.
4. Enable Redis-backed rate limiting at the platform gateway layer when `sdkwork-web-framework` production assembly requires it.
5. Points-recharge fulfillment is idempotent; payment callbacks may retry safely. Commit failure after wallet credit triggers automatic compensation debit and releases the `processing` reservation; operators may still replay via `payment_confirmations` if compensation fails.
6. Order cancellation and close (buyer or admin) close payment intents **before** mutating order status to avoid payable terminal orders.
7. Write commands require `Idempotency-Key` and `Sdkwork-Request-Hash`; replays against terminal order states return success without duplicate audit rows.

## Verification Before Release

```bash
pnpm verify
pnpm test:postgres:required   # CI uses ORDER_TEST_POSTGRES_URL
```

Contract drift is guarded by automated tests:

- OpenAPI ↔ Axum router mount (`app_openapi_routes`, `backend_openapi_routes`)
- HTTP manifest ↔ OpenAPI methods (`http_route_manifest` unit tests)
- Service contract ↔ HTTP manifest (`gateway-assembly` integration test)

## PC Surfaces

Operator actions require IAM permissions documented below. Set `VITE_SDKWORK_ACCESS_TOKEN` (and optional `VITE_SDKWORK_AUTH_TOKEN`) for the standalone PC build; without a token the shell shows a configuration hint instead of failing on opaque SDK errors.

| Path | Surface | SDK |
| --- | --- | --- |
| `/app/order` | Buyer order center | `@sdkwork/order-app-sdk` |
| `/admin/orders` | Operator order admin | `@sdkwork/order-backend-sdk` |

## IAM Permissions (backend)

| Permission | Scope |
| --- | --- |
| `commerce.afterSales.read` | List/retrieve after-sales requests |
| `commerce.afterSales.review` | Review after-sales requests |
| `commerce.orders.read` | Orders, shipments, events |
| `commerce.orders.manage` | Cancel/close orders; shipment package write |
