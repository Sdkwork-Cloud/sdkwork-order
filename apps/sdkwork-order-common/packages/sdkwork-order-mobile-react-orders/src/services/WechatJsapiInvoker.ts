/**
 * WeChat JSAPI payment invocation through the WeChat H5 bridge.
 *
 * Inside the WeChat app the cashier calls `WeixinJSBridge.invoke(
 * 'getBrandWCPayRequest', payload)` with the backend-provided JSAPI payload
 * (`jsapiPayload` in `PaymentSession.paymentParams`). The bridge may not be
 * ready yet, so callers wait for the `WeixinJSBridgeReady` events before
 * invoking.
 */

export interface WeixinJSBridgeLike {
  invoke(
    action: string,
    payload: Record<string, unknown>,
    callback: (result: Record<string, unknown>) => void,
  ): void;
}

export interface WechatJsapiInvokeResult {
  readonly errMsg?: string;
  readonly [key: string]: unknown;
}

export const WECHAT_JSAPI_UNAVAILABLE_ERROR = "WECHAT_JSAPI_UNAVAILABLE";

interface WeixinWindow extends Window {
  WeixinJSBridge?: WeixinJSBridgeLike;
}

/** Reads the WeChat bridge from the window global, if already injected. */
export function getWeixinJSBridge(): WeixinJSBridgeLike | undefined {
  return (window as WeixinWindow).WeixinJSBridge;
}

/**
 * Waits for the WeChat JS bridge to become available. Resolves with the
 * bridge once injected (via either readiness event) or with `null` after
 * `timeoutMs` so callers can fall back to QR/code_url rendering.
 */
export function waitForWeixinJSBridge(timeoutMs = 4000): Promise<WeixinJSBridgeLike | null> {
  const existing = getWeixinJSBridge();
  if (existing) {
    return Promise.resolve(existing);
  }
  return new Promise((resolve) => {
    let settled = false;
    const finish = (bridge: WeixinJSBridgeLike | null) => {
      if (settled) {
        return;
      }
      settled = true;
      document.removeEventListener("WeixinJSBridgeReady", handleReady);
      document.removeEventListener("onWeixinJSBridgeReady", handleReady);
      window.clearTimeout(timer);
      resolve(bridge);
    };
    const handleReady = () => finish(getWeixinJSBridge() ?? null);
    const timer = window.setTimeout(() => finish(getWeixinJSBridge() ?? null), timeoutMs);
    document.addEventListener("WeixinJSBridgeReady", handleReady, false);
    document.addEventListener("onWeixinJSBridgeReady", handleReady, false);
  });
}

/**
 * Invokes `getBrandWCPayRequest` with the backend JSAPI payload. Rejects
 * with `WECHAT_JSAPI_UNAVAILABLE_ERROR` when the bridge never became ready;
 * otherwise resolves with the bridge callback result.
 */
export async function invokeWechatJsapiPayment(
  payload: Record<string, unknown>,
): Promise<WechatJsapiInvokeResult> {
  const bridge = await waitForWeixinJSBridge();
  if (!bridge) {
    throw new Error(WECHAT_JSAPI_UNAVAILABLE_ERROR);
  }
  return new Promise((resolve) => {
    bridge.invoke("getBrandWCPayRequest", payload, (result) => {
      resolve((result ?? {}) as WechatJsapiInvokeResult);
    });
  });
}

/**
 * WeChat reports outcomes through `err_msg`:
 * - `get_brand_wcpay_request:cancel` — user cancelled
 * - `get_brand_wcpay_request:fail` — payment failed
 * - `get_brand_wcpay_request:ok` — accepted
 */
export function isWechatJsapiResultOk(result: WechatJsapiInvokeResult): boolean {
  const errMsg = String(result.errMsg ?? "").toLowerCase();
  return !errMsg || errMsg.includes("ok");
}

export function isWechatJsapiResultCancelled(result: WechatJsapiInvokeResult): boolean {
  return String(result.errMsg ?? "").toLowerCase().includes("cancel");
}
