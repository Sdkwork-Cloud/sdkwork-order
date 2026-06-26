# Order Technical Architecture
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-24

## 1. Architecture Overview

Describe the repository/application architecture.


## Capability stack

`sdkwork-order` owns the full **order** capability:

| Layer | Path |
| --- | --- |
| Domain (Rust) | `crates/sdkwork-commerce-order-service/` |
| SQL | `crates/sdkwork-commerce-order-repository-sqlx/` |
| HTTP routers | `crates/sdkwork-routes-order-*-api/` |
| API server | `crates/sdkwork-order-api-server/` |
| PC client | `apps/sdkwork-order-pc/` |
| Client facade | `packages/common/order/sdkwork-order-service/` |

## PC surface

```text
apps/sdkwork-order-pc/
  packages/sdkwork-order-pc-core/
  packages/sdkwork-order-pc-shell/
  packages/sdkwork-order-pc-order/    ← migrated from sdkwork-commerce-pc
```

Composition apps (`sdkwork-mall`, etc.) consume `@sdkwork/order-pc-order` via workspace paths — not a central commerce PC repo.

## Verification

```powershell
cd E:\sdkwork-space\sdkwork-order
pnpm verify
```

## Related docs

- [Commerce PC distribution](../../../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-pc-capability-distribution.md)
- [Commerce repository dissolution](../../../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-repository-dissolution.md)
