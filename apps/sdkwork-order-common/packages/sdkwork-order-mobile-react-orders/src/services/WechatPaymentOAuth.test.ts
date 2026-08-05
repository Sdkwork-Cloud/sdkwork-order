import { describe, expect, it } from "vitest";

import {
  buildCashierOAuthRedirect,
  parseWechatOAuthCallbackParams,
  stripWechatOAuthCallbackParams,
} from "./WechatPaymentOAuth";

describe("parseWechatOAuthCallbackParams", () => {
  it("reads the openid left by the IAM OAuth callback", () => {
    const params = new URLSearchParams("openid=o_abc123&scene=recharge");
    expect(parseWechatOAuthCallbackParams(params)).toEqual({ openid: "o_abc123" });
  });

  it("reads an error code when the exchange failed", () => {
    const params = new URLSearchParams("error=invalid_state");
    expect(parseWechatOAuthCallbackParams(params)).toEqual({ error: "invalid_state" });
  });

  it("returns an empty result when nothing was left behind", () => {
    expect(parseWechatOAuthCallbackParams(new URLSearchParams())).toEqual({});
  });
});

describe("stripWechatOAuthCallbackParams", () => {
  it("removes openid/error and keeps unrelated parameters", () => {
    const stripped = stripWechatOAuthCallbackParams(
      new URLSearchParams("openid=o_1&error=x&scene=recharge"),
    );
    expect(stripped.get("openid")).toBeNull();
    expect(stripped.get("error")).toBeNull();
    expect(stripped.get("scene")).toBe("recharge");
  });
});

describe("buildCashierOAuthRedirect", () => {
  it("builds the relative redirect from a hash-router location", () => {
    const location = {
      pathname: "/",
      search: "",
      hash: "#/orders/123/cashier",
    } as Pick<Location, "pathname" | "search" | "hash">;
    expect(buildCashierOAuthRedirect(location)).toBe("/#/orders/123/cashier");
  });

  it("preserves an existing query inside the hash route", () => {
    const location = {
      pathname: "/",
      search: "",
      hash: "#/orders/123/cashier?scene=recharge",
    } as Pick<Location, "pathname" | "search" | "hash">;
    expect(buildCashierOAuthRedirect(location)).toBe("/#/orders/123/cashier?scene=recharge");
  });
});
