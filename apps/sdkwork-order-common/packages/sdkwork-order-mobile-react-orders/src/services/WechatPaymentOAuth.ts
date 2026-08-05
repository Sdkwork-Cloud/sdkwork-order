/**
 * WeChat payment OAuth helpers for the H5 cashier.
 *
 * The cashier runs on a hash router, so the authorize redirect parameter is
 * the relative hash route (e.g. `/#/orders/123/cashier`). The IAM backend
 * appends the payer `openid` (or an `error` code) to that route's query and
 * redirects the payer back here; the cashier then creates a `wechat_jsapi`
 * payment with the openid.
 */

/** Host-injected channel that calls the IAM WeChat payment OAuth `start` endpoint. */
export interface WechatPaymentOAuthChannel {
  /**
   * Returns the WeChat authorize URL for the given cashier redirect path.
   * Implementations must validate the response shape (authorizeUrl) and
   * surface transport failures as rejected promises.
   */
  fetchAuthorizeUrl(redirect: string): Promise<string>;
}

export interface WechatOAuthCallbackParams {
  readonly openid?: string;
  readonly error?: string;
}

/** Reads the openid/error parameters left by the IAM OAuth callback. */
export function parseWechatOAuthCallbackParams(
  searchParams: URLSearchParams,
): WechatOAuthCallbackParams {
  const openid = searchParams.get("openid")?.trim() || undefined;
  const error = searchParams.get("error")?.trim() || undefined;
  return { openid, error };
}

/** Removes the OAuth callback parameters so the URL stays clean for retries. */
export function stripWechatOAuthCallbackParams(searchParams: URLSearchParams): URLSearchParams {
  const next = new URLSearchParams(searchParams);
  next.delete("openid");
  next.delete("error");
  return next;
}

/**
 * Builds the relative cashier redirect for the OAuth flow from the current
 * location. With a hash router the hash (including its own query) is the
 * part the backend appends the openid to.
 */
export function buildCashierOAuthRedirect(
  location: Pick<Location, "pathname" | "search" | "hash">,
): string {
  return `${location.pathname}${location.search}${location.hash}`;
}
