import { describe, expect, it } from "vitest";

import { createSdkworkIdempotencyParams } from "../src/idempotency.ts";

describe("idempotency params", () => {
  it("creates a UUID key for a logical command attempt", () => {
    expect(createSdkworkIdempotencyParams().idempotencyKey).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );
  });

  it("preserves an explicitly supplied retry key", () => {
    expect(createSdkworkIdempotencyParams("retry-key")).toEqual({
      idempotencyKey: "retry-key",
    });
  });
});
