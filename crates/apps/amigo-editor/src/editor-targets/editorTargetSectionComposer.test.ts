import { describe, expect, it } from "vitest";
import { composeSectionsForResolvedTarget } from "./editorTargetSectionComposer";
import type { ResolvedEditorTarget } from "./editorTargetTypes";
import type { EditorMetadataCatalogDto } from "../features/metadata/editorMetadataTypes";

describe("editorTargetSectionComposer", () => {
  it("composes sections from resolved target metadata traits", () => {
    const sections = composeSectionsForResolvedTarget(targetWithTraits(["Renderable2D"]), catalog);

    expect(sections.map((section) => section.id)).toEqual([
      "render2d.summary",
      "render2d.properties",
    ]);
  });

  it("skips unknown traits", () => {
    const sections = composeSectionsForResolvedTarget(targetWithTraits(["UnknownTrait"]), catalog);

    expect(sections).toEqual([]);
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
      editorSections: [
        {
          id: "render2d.summary",
          label: "Render2D",
          placement: "RightTop",
          description: "Render summary",
          priority: 100,
          defaultExpanded: true,
        },
        {
          id: "render2d.properties",
          label: "Render Properties",
          placement: "RightBottom",
          description: "Render properties",
          priority: 200,
          defaultExpanded: true,
        },
      ],
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
