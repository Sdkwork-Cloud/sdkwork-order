# sdkwork-order
repository-kind: application

SDKWork commerce **order** capability building-block repository (domain `commerce`).

- Standards: `../sdkwork-specs/README.md`
- Composition consumer: `../sdkwork-clawrouter/vendor/sdkwork-commerce (deleted)` (archived transitional platform snapshot)
- Domain service: `crates/sdkwork-order-service/`
- Repository SQL: `crates/sdkwork-commerce (deleted)-order-repository-sqlx/`
- HTTP API server: `crates/sdkwork-order-standalone-gateway/`

## Quick start

```bash
cargo test --workspace
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
