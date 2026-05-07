import { describe, expect, it } from "vitest";
import { uiNodeCapabilitiesForKind } from "./uiNodeCapabilities";

describe("uiNodeCapabilitiesForKind", () => {
  it("allows layout nodes to contain children", () => {
    expect(uiNodeCapabilitiesForKind("column").canAddChild).toBe(true);
    expect(uiNodeCapabilitiesForKind("panel").allowedChildren).toContain("button");
    expect(uiNodeCapabilitiesForKind("row").allowedChildren).toContain("text");
  });

  it("keeps text/button/image/spacer as leaves in MVP", () => {
    expect(uiNodeCapabilitiesForKind("text").canAddChild).toBe(false);
    expect(uiNodeCapabilitiesForKind("button").canAddChild).toBe(false);
    expect(uiNodeCapabilitiesForKind("image").canAddChild).toBe(false);
    expect(uiNodeCapabilitiesForKind("spacer").canAddChild).toBe(false);
  });

  it("marks buttons as trigger/navigation capable", () => {
    const caps = uiNodeCapabilitiesForKind("button");
    expect(caps.canTrigger).toBe(true);
    expect(caps.canNavigate).toBe(true);
  });
});
