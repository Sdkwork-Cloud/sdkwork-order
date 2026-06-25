import { sdkworkOrderPcRuntimeIdentity } from "@sdkwork/order-pc-core";

export function OrderAppShell() {
  return (
    <main className="order-shell">
      <section className="order-card">
        <h1>SDKWork Order</h1>
        <p>{sdkworkOrderPcRuntimeIdentity.appKey}</p>
        <p>Order capability PC surface — aligned with sdkwork-specs building-block model.</p>
      </section>
    </main>
  );
}
