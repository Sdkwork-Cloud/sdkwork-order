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
| Domain (Rust) | `crates/sdkwork-order-service/` |
| SQL | `crates/sdkwork-commerce (deleted)-order-repository-sqlx/` |
| HTTP routers | `crates/sdkwork-routes-order-*-api/` |
| API server | `crates/sdkwork-order-standalone-gateway/` |
| PC client | `apps/sdkwork-order-pc/` |
| Client facade | `apps/sdkwork-order-common/packages/sdkwork-order-service/` |

## PC surface

```text
apps/sdkwork-order-pc/
  packages/sdkwork-order-pc-core/
  packages/sdkwork-order-pc-shell/
  packages/sdkwork-order-pc-order/    ← migrated from sdkwork-commerce (deleted)-pc
```

Composition apps (`sdkwork-mall`, etc.) consume `@sdkwork/order-pc-order` via workspace paths — not a central commerce PC repo.

## Verification

```powershell
cd E:\sdkwork-space\sdkwork-order
pnpm verify
```

## Related docs

- Commerce repository dissolution: `../../sdkwork-specs/MIGRATION_SPEC.md` §8
