# Commerce Checkout and Payment Architecture

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-07-07  
Machine contracts: `specs/commerce-checkout-topology.spec.json`, `specs/commerce-payment-webhook.spec.json`

## 1. Capability Boundaries

| Capability | Repository | Role |
| --- | --- | --- |
| **Order** | `sdkwork-order` | Unified order center: checkout, pay orchestration, **PSP webhook ingestion**, payment settlement, fulfillment sagas |
| **Payment** | `sdkwork-payment` | Payment executor: intents, attempts, provider channels, refunds; **webhook event persistence via port only** |

**Dependency direction:** `sdkwork-order` → `sdkwork-payment` (in-process stores and `sdkwork-payment-providers`). **Payment MUST NOT depend on Order** (no HTTP callbacks, no order service imports in route crates).

## 2. End-to-End Flows

### 2.1 Product checkout (mall / physical goods)

```mermaid
sequenceDiagram
    participant Client
    participant Order as order-app-api
    participant Payment as payment (in-process port)
    participant Cashier as Cashier UI
    participant Fulfill as fulfillment/shipment

    Client->>Order: checkout.sessions.create
    Client->>Order: checkout.sessions.orders.create
    Client->>Order: orders.pay
    Order->>Payment: pay_owner_order
    Payment-->>Client: paymentParams.cashierUrl
    Client->>Cashier: open cashierUrl
    Cashier->>Order: PSP webhook POST .../orders/payments/webhooks/{provider}
    Order->>Payment: ingest webhook (port)
    Order->>Order: settle + fulfill (subject=product)
    Order->>Fulfill: shipment / virtual entitlement
```

### 2.2 Points recharge

```mermaid
sequenceDiagram
    participant Client
    participant Order as order-app-api
    participant Payment as payment port
    participant Account as account-backend

    Client->>Order: recharges.orders.create
    Client->>Order: orders.pay
    Order->>Payment: pay_owner_order
    Payment-->>Client: paymentParams.cashierUrl
    Note over Order: PSP webhook on order gateway
    Order->>Payment: ingest webhook
    Order->>Order: settle_owner_order_after_payment_success
    Order->>Account: ledger credit (Bearer service token)
```

### 2.3 Provider webhook (production)

```mermaid
sequenceDiagram
    participant PSP as Payment provider
    participant Order as order-app-api webhook
    participant Payment as payment repository port

    PSP->>Order: POST /app/v3/api/orders/payments/webhooks/{provider}
    Order->>Order: verify + normalize (payment-providers)
    Order->>Payment: ingest_provider_webhook
    Payment-->>Order: attempt context + succeeded
    Order->>Order: in-process settlement saga
```

Legacy payment gateway path `POST /app/v3/api/payments/webhooks/{provider}` returns **410 Gone** with migration guidance.

Manual operator replay: `POST /backend/v3/api/orders/{orderId}/payment_confirmations`.

### 2.4 Membership purchase

Order center **creates** `commerce_order` with `subject=membership` (checkout or membership-subject order create — parallel to `recharges.orders.create`). Membership **must not** insert `commerce_order` in production paths.

```mermaid
sequenceDiagram
    participant Client
    participant Order as order-app-api
    participant Membership as membership-app-api
    participant Payment as payment port

    Client->>Order: memberships.orders.create
    Client->>Order: orders.pay
    Order->>Payment: pay_owner_order
    Note over Order: PSP webhook on order gateway
    Order->>Payment: ingest webhook
    Order->>Order: settle_owner_order_after_payment_success
    Order->>Membership: MembershipPurchaseFulfillmentPort
```

**Dependency:** `sdkwork-order` → `sdkwork-membership` via fulfillment port at gateway assembly (same pattern as order → account for points recharge). Payment remains a foundation module with no order or membership dependencies.

Authority: `../sdkwork-membership/specs/COMMERCE_ORDER_BOUNDARY_SPEC.md`, `../sdkwork-membership/specs/commerce-order-membership-boundary.spec.json`.

## 3. Cashier URL Contract

Cashier deep-links are built by `sdkwork-utils-rust`:

- `commerce_cashier_base_url()` — env `SDKWORK_COMMERCE_CASHIER_BASE_URL`, default `https://im.sdkwork.com/cashier`
- `commerce_cashier_scene(order_subject)` — maps `points_recharge` → `recharge`, `product` → `checkout`
- `build_commerce_cashier_url(scene, order_id, out_trade_no)`

`orders.pay` and recharge pay outcomes expose:

| `paymentParams` key | Meaning |
| --- | --- |
| `cashierUrl` | Full deep-link for cashier UI |
| `nextAction` | Always `cashier` when redirect is required |
| `orderSn` | Business order number (`order_no`) |
| `cashierScene` | `recharge`, `checkout`, or `virtual` |
| `qrCodePayload` | Same as `cashierUrl` for scan-to-pay |

**Wire note:** `orderId` in the URL is the business `order_no`, not the internal UUID.

## 4. Client Architecture by Platform

All application packages **must** consume composed SDKs (`@sdkwork/order-app-sdk`, `@sdkwork/payment-app-sdk`). Raw HTTP and generator transport package names are forbidden per `APP_SDK_INTEGRATION_SPEC.md` section 9.

### 4.1 PC (React / Vite)

| Concern | Implementation |
| --- | --- |
| Order center (standalone) | `apps/sdkwork-order-pc` — list, detail, pay, cancel |
| Order center (composed) | `sdkwork-mall-pc-checkout`, `sdkwork-account-pc-wallet` embed `@sdkwork/order-app-sdk` |
| Checkout | `sdkwork-mall-pc-checkout` — `checkout.*` + `recharges.*` |
| Cashier | `sdkwork-payment-pc` or host shell route; navigate to `paymentParams.cashierUrl` after `orders.pay` |
| Service wiring | `apps/sdkwork-order-common/packages/sdkwork-order-service` facade over SDK ports |

### 4.2 Backend / service-to-service

| Call | Auth |
| --- | --- |
| Order → account credit | `SDKWORK_ACCESS_TOKEN` (Bearer) |

PSP notify URLs MUST target the **order gateway**, not payment:

`{ORDER_PAYMENT_WEBHOOK_BASE_URL}/app/v3/api/orders/payments/webhooks/{providerCode}`

## 5. API Surface Map

| Operation group | App prefix | Primary SDK |
| --- | --- | --- |
| Orders | `/app/v3/api/orders` | `@sdkwork/order-app-sdk` → `orders.*` |
| Payment webhooks | `/app/v3/api/orders/payments/webhooks` | order gateway (PSP-facing) |
| Recharges | `/app/v3/api/recharges` | `@sdkwork/order-app-sdk` → `recharges.*` |
| Checkout | `/app/v3/api/checkout` | `@sdkwork/order-app-sdk` → `checkout.*` |
| Payments (execute) | in-process port from order | `@sdkwork/payment-app-sdk` for cashier reads |
| Admin orders | `/backend/v3/api/orders` | `@sdkwork/order-backend-sdk` |

## 6. Idempotency and Pagination

- Pay, webhook settlement, and fulfillment commands require `requestNo` + `idempotencyKey` headers per OpenAPI where applicable.
- List endpoints use `SdkWorkListQuery` (`page`, `page_size`; default 20, max 200).
- Success envelope: `SdkWorkApiResponse` with `code: 0`, `data`, `traceId`.

## 7. Related Specs

- Payment webhook: `specs/commerce-payment-webhook.spec.json`
- Recharge boundary: `specs/commerce-recharge.spec.json`
- Payment boundary: `../sdkwork-payment/specs/commerce-boundary.spec.json`
- Integrator guide: `docs/guides/integrator/README.md`
