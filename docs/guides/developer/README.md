# Developer Guide

Status: active  
Updated: 2026-07-07

## Local Setup

```bash
pnpm install
pnpm start          # standalone gateway (default 0.0.0.0:18093)
pnpm dev            # PC client at apps/sdkwork-order-pc
```

Set `ORDER_CORS_ALLOW_ORIGINS` for browser clients. Use `ORDER_READ_MODEL_LENIENT=1` only for local scaffolding without full commerce DDL.

## Verification

```bash
pnpm verify
pnpm test:node      # OpenAPI ↔ router ↔ SDK authority sync
pnpm test:postgres  # optional Postgres parity when ORDER_TEST_POSTGRES_URL is set
```

## Write Command Headers

Integrators and PC packages must send `Idempotency-Key` and `Sdkwork-Request-Hash` on every idempotent write. See [integrator guide](../integrator/README.md) and [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md).

## Key Paths

| Area | Path |
| --- | --- |
| App routers | `crates/sdkwork-routes-order-app-api/` |
| Backend routers | `crates/sdkwork-routes-order-backend-api/` |
| Domain service | `crates/sdkwork-order-service/` |
| TS service facade | `apps/sdkwork-order-common/packages/sdkwork-order-service/` |
| OpenAPI authorities | `apis/app-api/order/`, `apis/backend-api/order/` |

After OpenAPI edits: `pnpm sync:openapi` then `pnpm sdk:generate` / `pnpm sdk:generate:backend`.

Authority: [AGENTS.md](../../AGENTS.md), `sdkwork-specs/README.md`.
