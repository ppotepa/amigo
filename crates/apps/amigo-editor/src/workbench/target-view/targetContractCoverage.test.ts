import { describe, expect, it, beforeEach } from "vitest";
import { registerBuiltinTargetContracts, resetBuiltinTargetContractsForTests } from "./registerBuiltinTargetContracts";
import { allTargetContracts, clearTargetContractsForTests } from "./targetViewRegistry";

describe("target contract coverage", () => {
  beforeEach(() => {
    clearTargetContractsForTests();
    resetBuiltinTargetContractsForTests();
  });

  it("registers non-generic contracts for first-class target kinds", () => {
    registerBuiltinTargetContracts();

    const ids = allTargetContracts().map((contract) => contract.id);

    expect(ids).toContain("scene");
    expect(ids).toContain("scene-entity");
    expect(ids).toContain("asset");
    expect(ids).toContain("component");
    expect(ids).toContain("project-file");
  });

  it("does not register first-class target kinds as generic fallbacks", () => {
    registerBuiltinTargetContracts();

    const entityContract = allTargetContracts().find((contract) => contract.kind === "sceneEntity");
    const assetContract = allTargetContracts().find((contract) => contract.kind === "asset");
    const componentContract = allTargetContracts().find((contract) => contract.kind === "component");
    const fileContract = allTargetContracts().find((contract) => contract.kind === "projectFile");

    expect(entityContract?.id).toBe("scene-entity");
    expect(assetContract?.id).toBe("asset");
    expect(componentContract?.id).toBe("component");
    expect(fileContract?.id).toBe("project-file");
  });
});
