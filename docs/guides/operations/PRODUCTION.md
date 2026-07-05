# Production Operations

Status: active  
Updated: 2026-07-06

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
| `SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN` | Production | Account wallet credit for points-recharge fulfillment |
| `RUST_LOG` | No | e.g. `info,order.bootstrap=info,order.runtime=info` |

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
5. Points-recharge fulfillment is idempotent; payment callbacks may retry safely.

## Verification Before Release

```bash
pnpm verify
pnpm test:postgres:required   # CI uses ORDER_TEST_POSTGRES_URL
```

## PC Surfaces

| Path | Surface | SDK |
| --- | --- | --- |
| `/app/order` | Buyer order center | `@sdkwork/order-app-sdk` |
| `/admin/orders` | Operator order admin | `@sdkwork/order-backend-sdk` |

Operator actions require IAM permissions `commerce.orders.read` and `commerce.orders.manage`.
