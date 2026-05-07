import { describe, expect, it } from "vitest";
import { uiNodeCapabilitiesForKind } from "./uiNodeCapabilities";

describe("uiNodeCapabilitiesForKind", () => {
  it("allows layout containers to contain children", () => {
    expect(uiNodeCapabilitiesForKind("column").canAddChild).toBe(true);
    expect(uiNodeCapabilitiesForKind("panel").allowedChildren).toContain("button");
  });

  it("does not allow text and button nodes to contain children in MVP", () => {
    expect(uiNodeCapabilitiesForKind("text").canAddChild).toBe(false);
    expect(uiNodeCapabilitiesForKind("button").canAddChild).toBe(false);
  });

  it("marks buttons as trigger-capable", () => {
    const caps = uiNodeCapabilitiesForKind("button");
    expect(caps.canTrigger).toBe(true);
    expect(caps.canNavigate).toBe(true);
  });
});
