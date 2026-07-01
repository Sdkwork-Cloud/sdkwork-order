# sdkwork-order Component Specs

Local specification index for the order capability (unified `commerce_order`, including recharge).

## Spec map

| Document | Purpose |
| --- | --- |
| [component.spec.json](./component.spec.json) | Workspace manifest |
| [RECHARGE_ORDER_SPEC.md](./RECHARGE_ORDER_SPEC.md) | Recharge packages, recharge orders, unified order center |
| [commerce-recharge.spec.json](./commerce-recharge.spec.json) | Machine-readable recharge contract + migration from payment |

## Sibling specs

| Repository | Entry |
| --- | --- |
| `sdkwork-account` | `specs/COMMERCE_BOUNDARY_SPEC.md` |
| `sdkwork-payment` | `specs/PAYMENT_EXECUTOR_SPEC.md` |

## Verification

```powershell
cd ..\sdkwork-order
pnpm verify
cargo test --workspace
```
