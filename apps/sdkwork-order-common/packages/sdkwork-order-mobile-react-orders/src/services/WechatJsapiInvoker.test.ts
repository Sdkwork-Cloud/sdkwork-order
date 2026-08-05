import { describe, expect, it } from "vitest";

import {
  getWeixinJSBridge,
  isWechatJsapiResultCancelled,
  isWechatJsapiResultOk,
  waitForWeixinJSBridge,
  type WeixinJSBridgeLike,
} from "./WechatJsapiInvoker";

describe("isWechatJsapiResultOk / isWechatJsapiResultCancelled", () => {
  it("treats an empty or ok errMsg as success", () => {
    expect(isWechatJsapiResultOk({})).toBe(true);
    expect(isWechatJsapiResultOk({ errMsg: "get_brand_wcpay_request:ok" })).toBe(true);
  });

  it("flags cancel and fail outcomes", () => {
    expect(isWechatJsapiResultOk({ errMsg: "get_brand_wcpay_request:cancel" })).toBe(false);
    expect(isWechatJsapiResultOk({ errMsg: "get_brand_wcpay_request:fail" })).toBe(false);
    expect(isWechatJsapiResultCancelled({ errMsg: "get_brand_wcpay_request:cancel" })).toBe(true);
    expect(isWechatJsapiResultCancelled({ errMsg: "get_brand_wcpay_request:fail" })).toBe(false);
  });
});

describe("getWeixinJSBridge / waitForWeixinJSBridge", () => {
  it("resolves immediately when the bridge is already injected", async () => {
    const bridge: WeixinJSBridgeLike = {
      invoke: () => undefined,
    };
    Object.defineProperty(window, "WeixinJSBridge", {
      configurable: true,
      value: bridge,
    });
    expect(getWeixinJSBridge()).toBe(bridge);
    await expect(waitForWeixinJSBridge(50)).resolves.toBe(bridge);
  });

  it("resolves with null when the bridge never becomes ready", async () => {
    Object.defineProperty(window, "WeixinJSBridge", {
      configurable: true,
      value: undefined,
    });
    await expect(waitForWeixinJSBridge(20)).resolves.toBeNull();
  });
});
