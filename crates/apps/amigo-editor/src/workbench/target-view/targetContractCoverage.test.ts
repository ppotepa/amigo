import { describe, expect, it, beforeEach } from "vitest";
import { registerBuiltinTargetContracts, resetBuiltinTargetContractsForTests } from "./registerBuiltinTargetContracts";
import { allTargetContracts, clearTargetContractsForTests } from "./targetViewRegistry";

describe("target contract coverage", () => {
  beforeEach(() => {
    clearTargetContractsForTests();
    resetBuiltinTargetContractsForTests();
  });

  it("registers non-generic contracts for scene and sceneEntity", () => {
    registerBuiltinTargetContracts();

    const ids = allTargetContracts().map((contract) => contract.id);

    expect(ids).toContain("scene");
    expect(ids).toContain("scene-entity");
  });

  it("does not register sceneEntity as generic fallback", () => {
    registerBuiltinTargetContracts();

    const entityContract = allTargetContracts().find((contract) => contract.kind === "sceneEntity");

    expect(entityContract?.id).toBe("scene-entity");
  });
});
