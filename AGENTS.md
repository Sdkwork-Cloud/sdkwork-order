# Repository Guidelines

## SDKWORK Soul

Read `../sdkwork-specs/SOUL.md` before executing tasks in this root.

## Capability Identity

- Domain: `commerce`
- Capability: `order`
- Table prefix: `commerce_`
- App API prefix: `/app/v3/api/orders`
- Backend API prefix: `/backend/v3/api/orders`

This repo owns the full order capability stack: Rust services, database, APIs, SDKs, and **PC client surface** at `apps/sdkwork-order-pc/` (see `sdkwork-shop/apps/sdkwork-shop-pc/` as template).

PC packages migrate from `sdkwork-commerce/apps/sdkwork-commerce-pc/packages/sdkwork-commerce-pc-order*` — see `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-pc-capability-distribution.md`.

Do not depend on `sdkwork-commerce` monolith crates after dissolution.

## Verification

```bash
cargo test --workspace
pnpm install && pnpm verify   # PC surface
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)
