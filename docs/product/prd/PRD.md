# Order PRD

Status: active
Owner: SDKWork maintainers
Application: order
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Platform split alignment (commerce T0): `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`

## 1. Background And Problem

Order, checkout, fulfillment, shipment, and after-sales lifecycles must scale independently from payment and catalog capabilities.

This repository is a **T1 commerce capability building block**. `sdkwork-commerce` remains the T0 composition layer (gateway, IAM wrappers, composed SDK). This repository owns domain logic, persistence, and HTTP route builders for the **order** capability.

## 2. Target Users

Buyers, merchant operators, fulfillment staff, and commerce integrators.

## 3. Goals And Non-Goals

### Goals

- Own order lifecycle domain, SQL repositories, and app HTTP routers.
- Expose checkout, fulfillment, shipment, and after-sales routes via `build_*_router` exports.

### Non-Goals

- Payment intent/refund SQL ownership (owned by payment capability).
- IAM middleware in capability routers.

## 4. Scope

- Order create/list/detail/cancel flows.
- Checkout session lifecycle.
- Fulfillment, shipment tracking, after-sales requests.
- Order repository SQL for order lifecycle tables only.

Primary API prefixes:

- App: `/app/v3/api/orders`
- Backend: `/backend/v3/api/orders`

Migration status: **complete**.

## 5. User Scenarios

- A buyer creates a checkout session, places an order, and tracks fulfillment status.
- Commerce T0 wraps order routers with request identity; handlers remain capability-owned.

## 6. Success Metrics

- Order integration tests pass in commerce api-server via thin wrappers.
- `sdkwork-commerce-order-repository-sqlx` is the sole order SQL owner.

## 7. Phases

- Phase 1 (complete): SQL + five app routers owned in sdkwork-order.
- Phase 2 (complete): payment_intent/refund SQL owned by payment repository; order validates via one-way dependency.
- Phase 3 (complete): pay_owner_order and cancel payment side-effects owned by payment repository; order repo is order-table only.

## 8. Linked Requirements

- Commerce capability split alignment: `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions


