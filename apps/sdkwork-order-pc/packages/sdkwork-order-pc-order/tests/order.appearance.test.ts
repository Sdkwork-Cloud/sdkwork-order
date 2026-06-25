import { describe, expect, it } from "vitest";
import {
  createSdkworkOrderBackdropStyle,
  createSdkworkOrderHeroStyle,
  createSdkworkOrderPanelStyle,
  createSdkworkOrderToneStyle,
} from "../src";

describe("sdkwork-order-pc-order appearance", () => {
  it("creates token-driven tone and panel styles", () => {
    const toneStyle = createSdkworkOrderToneStyle("warning");
    const panelStyle = createSdkworkOrderPanelStyle("accent");

    expect(String(toneStyle.backgroundColor)).toContain("var(--sdk-color-state-warning)");
    expect(String(panelStyle.backgroundImage)).toContain("var(--sdk-color-brand-accent)");
  });

  it("creates sdkwork-style backdrop and hero gradients", () => {
    const backdropStyle = createSdkworkOrderBackdropStyle();
    const heroStyle = createSdkworkOrderHeroStyle();

    expect(String(backdropStyle.backgroundImage)).toContain("var(--sdk-color-brand-primary)");
    expect(String(heroStyle.backgroundImage)).toContain("linear-gradient");
    expect(String(heroStyle.backgroundImage)).not.toContain("#111827");
  });
});
