import { describe, expect, it, beforeEach } from "vitest";
import type { EditorTargetKind, ResolvedEditorTarget } from "../../editor-targets";
import { entityTargetContract } from "../../features/entity/target";
import { sceneTargetContract } from "../../features/scene/target";
import { registerBuiltinTargetContracts, resetBuiltinTargetContractsForTests } from "./registerBuiltinTargetContracts";
import { resolveTargetContract } from "./resolveTargetContract";
import { allTargetContracts, clearTargetContractsForTests } from "./targetViewRegistry";

describe("target view registry", () => {
  beforeEach(() => {
    clearTargetContractsForTests();
    resetBuiltinTargetContractsForTests();
  });

  it("registers the scene target contract", () => {
    registerBuiltinTargetContracts();

    expect(allTargetContracts()).toContain(sceneTargetContract);
  });

  it("registers the entity target contract", () => {
    registerBuiltinTargetContracts();

    expect(allTargetContracts()).toContain(entityTargetContract);
  });

  it("resolves scene targets to the scene target contract", () => {
    registerBuiltinTargetContracts();

    expect(resolveTargetContract(target("scene"))).toBe(sceneTargetContract);
  });

  it("registers generic target contracts for non-scene kinds", () => {
    registerBuiltinTargetContracts();

    expect(resolveTargetContract(target("asset"))?.kind).toBe("asset");
    expect(resolveTargetContract(target("component"))?.kind).toBe("component");
  });

  it("resolves scene entity targets to the entity target contract", () => {
    registerBuiltinTargetContracts();

    expect(resolveTargetContract(target("sceneEntity"))).toBe(entityTargetContract);
  });

  it("keeps scene contract more specific than generic contracts", () => {
    registerBuiltinTargetContracts();

    const sceneContract = resolveTargetContract(target("scene"));

    expect(sceneContract).toBe(sceneTargetContract);
    expect(sceneContract?.id).toBe("scene");
  });

  it("is idempotent", () => {
    registerBuiltinTargetContracts();
    registerBuiltinTargetContracts();

    const ids = allTargetContracts().map((contract) => contract.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

function target(kind: EditorTargetKind): ResolvedEditorTarget {
  return {
    ref: targetRef(kind),
    status: "resolved",
    descriptor: {
      kind,
      label: kind,
      icon: "box",
      breadcrumbs: [],
      canInspect: true,
      canOpen: false,
      canReveal: false,
      selectionKind: "none",
      actions: [],
    },
    selection: { kind: "empty" },
    capabilities: [],
    metadataTraits: [],
    metadataRefs: [],
    documentRefs: [],
    relatedTargets: [],
    diagnostics: [],
    breadcrumbs: [],
    actions: [],
  };
}

function targetRef(kind: EditorTargetKind): ResolvedEditorTarget["ref"] {
  switch (kind) {
    case "scene":
      return { kind, sceneId: "scene-main" };
    case "asset":
      return { kind, assetKey: "asset" };
    case "component":
      return { kind, sceneId: "scene-main", componentIndex: 0, componentType: "Transform2D" };
    case "sceneEntity":
      return { kind, sceneId: "scene-main", entityId: "entity" };
    case "projectFile":
    case "script":
      return { kind, path: "path" };
    case "mod":
      return { kind, modId: "mod" };
    case "projectNode":
      return { kind, nodeId: "node", nodeKind: "folder" };
    case "uiDocument":
      return { kind, sceneId: "scene-main", entityId: "entity", componentIndex: 0 };
    case "uiNode":
      return { kind, sceneId: "scene-main", entityId: "entity", componentIndex: 0, nodePath: "root" };
    case "diagnostic":
      return { kind, diagnosticId: "diagnostic" };
    case "capability":
      return { kind, capabilityId: "capability" };
    case "dependency":
      return { kind, dependencyId: "dependency" };
  }
}
