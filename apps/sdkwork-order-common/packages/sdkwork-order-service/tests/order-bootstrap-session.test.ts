import { createTokenManager } from "@sdkwork/sdk-common";
import { afterEach, describe, expect, it } from "vitest";

import {
  bootstrapSdkworkOrderAppService,
  configureSdkworkOrderAppServiceProvider,
  configureSdkworkOrderSessionTokenProvider,
  getSdkworkOrderSessionTokens,
} from "../src/index.ts";

describe("bootstrapSdkworkOrderAppService", () => {
  afterEach(() => {
    configureSdkworkOrderAppServiceProvider(null);
    configureSdkworkOrderSessionTokenProvider(null);
  });

  it("shares dynamic TokenManager credentials with order-owned services", () => {
    const tokenManager = createTokenManager({
      accessToken: "access-token-1",
      authToken: "auth-token-1",
      refreshToken: "refresh-token-1",
    });

    bootstrapSdkworkOrderAppService({
      baseUrl: "http://127.0.0.1:3901",
      tokenManager,
    });

    expect(getSdkworkOrderSessionTokens()).toEqual({
      accessToken: "access-token-1",
      authToken: "auth-token-1",
      refreshToken: "refresh-token-1",
    });

    tokenManager.setTokens({ accessToken: "access-token-2" });
    expect(getSdkworkOrderSessionTokens()).toEqual({
      accessToken: "access-token-2",
      authToken: undefined,
      refreshToken: undefined,
    });
  });

  it("registers directly supplied bootstrap credentials", () => {
    bootstrapSdkworkOrderAppService({
      accessToken: "access-token",
      authToken: "auth-token",
      baseUrl: "http://127.0.0.1:3901",
    });

    expect(getSdkworkOrderSessionTokens()).toEqual({
      accessToken: "access-token",
      authToken: "auth-token",
      refreshToken: undefined,
    });
  });
});
