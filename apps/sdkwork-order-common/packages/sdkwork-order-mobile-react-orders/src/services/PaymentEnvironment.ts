/**
 * Payment environment detection for the H5 cashier.
 *
 * The cashier behaves differently inside the Alipay app, the WeChat app and
 * a plain mobile browser: only Alipay payment is offered inside Alipay
 * (WAP redirect), only WeChat payment inside WeChat (JSAPI via OAuth), and
 * the full method list with QR scanning in a browser.
 */

export type PaymentEnvironment = "alipay" | "wechat" | "browser";

const ALIPAY_UA_PATTERN = /AlipayClient/i;
const WECHAT_UA_PATTERN = /MicroMessenger/i;

/**
 * Detects the payment environment from a user agent string. The browser
 * global is only read when `userAgent` is omitted, so tests can inject a
 * fixed UA.
 */
export function detectPaymentEnvironment(userAgent?: string): PaymentEnvironment {
  const ua = userAgent ?? (typeof navigator !== "undefined" ? navigator.userAgent : "");
  if (ALIPAY_UA_PATTERN.test(ua)) {
    return "alipay";
  }
  if (WECHAT_UA_PATTERN.test(ua)) {
    return "wechat";
  }
  return "browser";
}

/** True when the cashier runs inside the Alipay app webview. */
export function isAlipayEnvironment(environment: PaymentEnvironment): boolean {
  return environment === "alipay";
}

/** True when the cashier runs inside the WeChat app webview. */
export function isWechatEnvironment(environment: PaymentEnvironment): boolean {
  return environment === "wechat";
}
