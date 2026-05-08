import { describe, expect, it } from "vitest";
import { composeSectionsForResolvedTarget } from "./editorTargetSectionComposer";
import type { ResolvedEditorTarget } from "./editorTargetTypes";
import type { EditorMetadataCatalogDto } from "../features/metadata/editorMetadataTypes";

describe("editorTargetSectionComposer", () => {
  it("composes frontend-owned sections from metadata traits", () => {
    const sections = composeSectionsForResolvedTarget(targetWithTraits(["Renderable2D"]), catalog);
    expect(sections.map((section) => section.id)).toEqual([
      "render2d.summary",
      "render2d.details",
    ]);
    expect(sections.every((section) => section.placement === "summary" || section.placement === "details")).toBe(true);
  });

  it("creates fallback details section for unknown trait", () => {
    const sections = composeSectionsForResolvedTarget(targetWithTraits(["UnknownTrait"]), catalog);
    expect(sections).toHaveLength(1);
    expect(sections[0].id).toBe("UnknownTrait.details");
    expect(sections[0].placement).toBe("details");
  });
});

const catalog = {
  components: [],
  metadataTraits: [
    {
      kind: "Renderable2D",
      label: "Renderable 2D",
      description: "Render content",
      appliesTo: ["Entity", "Component"],
      expectedYamlShapes: [],
      propertyGroups: [],
      controls: [],
      patchOps: [],
      diagnostics: [],
    },
  ],
} satisfies EditorMetadataCatalogDto;

function targetWithTraits(metadataTraits: string[]): ResolvedEditorTarget {
  return {
    ref: { kind: "scene", sceneId: "test-scene" },
    status: "resolved",
    descriptor: {
      kind: "scene",
      label: "Test Scene",
      icon: "Scene",
      breadcrumbs: [],
      canOpen: false,
      canReveal: false,
      canInspect: true,
      selectionKind: "scene",
      actions: [],
    },
    contextProfile: {
      primary: [],
      secondary: [],
      defaultAction: "inspect",
    },
    selection: { kind: "empty" },
    capabilities: [],
    metadataTraits,
    metadataRefs: [],
    documentRefs: [],
    relatedTargets: [],
    diagnostics: [],
    breadcrumbs: [],
    actions: [],
  };
}
