import { describe, expect, it } from "vitest";
import {
  type CreateOrderWorkspaceManifestOptions,
  createOrderRouteIntent,
  createOrderWorkspaceManifest,
  orderPackageMeta,
  type SdkworkOrderMessagesOverrides,
} from "../src";

describe("sdkwork-order-pc-order headless contract", () => {
  it("creates reusable order manifests and route intents", () => {
    expect(orderPackageMeta).toMatchObject({
      domain: "commerce",
      package: "@sdkwork/order-pc-order",
    });

    expect(
      createOrderWorkspaceManifest({
        title: "Orders",
      }),
    ).toMatchObject({
      capability: "order",
      packageNames: ["@sdkwork/order-pc-order"],
      routePath: "/orders",
      title: "Orders",
    });

    expect(
      createOrderRouteIntent({
        orderId: "ORDER-3",
      }),
    ).toEqual({
      focusWindow: true,
      orderId: "ORDER-3",
      route: "/orders?orderId=ORDER-3",
      source: "order-workspace",
      type: "order-route-intent",
    });
  });

  it("localizes order workspace manifest defaults through the copy seam", () => {
    expect(
      createOrderWorkspaceManifest({
        locale: "en-US",
        messages: {
          manifest: {
            description: "Localized order manifest description",
            title: "Localized order title",
          },
        } satisfies SdkworkOrderMessagesOverrides,
      } satisfies CreateOrderWorkspaceManifestOptions),
    ).toMatchObject({
      description: "Localized order manifest description",
      title: "Localized order title",
    });
  });
});
