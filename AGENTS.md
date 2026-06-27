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

PC packages migrate from the dissolved `sdkwork-commerce` monolith — see `../sdkwork-specs/MIGRATION_SPEC.md` §8 for the commerce repository dissolution plan.

Do not depend on `sdkwork-commerce` monolith crates; the repository has been dissolved.

## Verification

```bash
cargo test --workspace
pnpm install && pnpm verify   # PC surface
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)
