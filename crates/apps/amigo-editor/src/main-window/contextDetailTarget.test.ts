import { describe, expect, it } from "vitest";
import { defaultDetailsTabForTarget } from "./contextDetailTarget";
import type { ResolvedEditorTarget } from "../editor-targets";

function target(kind: ResolvedEditorTarget["ref"]["kind"], diagnostics: unknown[] = []) {
  return {
    ref: { kind },
    diagnostics,
  } as Pick<ResolvedEditorTarget, "ref" | "diagnostics">;
}

describe("defaultDetailsTabForTarget", () => {
  it("uses scene info for empty and scene targets", () => {
    expect(defaultDetailsTabForTarget(null)).toBe("scene-info");
    expect(defaultDetailsTabForTarget(target("scene"))).toBe("scene-info");
  });

  it("uses file info for file-like targets", () => {
    expect(defaultDetailsTabForTarget(target("projectFile"))).toBe("file-info");
    expect(defaultDetailsTabForTarget(target("script"))).toBe("file-info");
  });

  it("uses diagnostics before generic properties", () => {
    expect(defaultDetailsTabForTarget(target("component", [{}]))).toBe("diagnostics");
    expect(defaultDetailsTabForTarget(target("component"))).toBe("properties");
  });
});
