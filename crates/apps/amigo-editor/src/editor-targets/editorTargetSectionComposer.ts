import type { EditorMetadataCatalogDto } from "../features/metadata/editorMetadataTypes";
import { findMetadataTraitDescriptor } from "../features/metadata/editorMetadataTypes";
import type { ResolvedEditorTarget } from "./editorTargetTypes";

export type EditorTargetSectionPlacement =
  | "summary"
  | "details"
  | "viewport"
  | "contextMenu";

export type EditorTargetComposedSection = {
  id: string;
  label: string;
  placement: EditorTargetSectionPlacement;
  traitKind: string;
  description: string;
  priority: number;
  defaultExpanded: boolean;
};

type TraitSectionRule = {
  traitKind: string;
  summary?: Omit<EditorTargetComposedSection, "traitKind" | "placement">;
  details?: Omit<EditorTargetComposedSection, "traitKind" | "placement">;
  viewport?: Omit<EditorTargetComposedSection, "traitKind" | "placement">;
  contextMenu?: Omit<EditorTargetComposedSection, "traitKind" | "placement">;
};

const TRAIT_SECTION_RULES: readonly TraitSectionRule[] = [
  {
    traitKind: "SceneDocument",
    summary: section("scene.document.summary", "Scene Document", "Scene document summary.", 10, true),
    details: section("scene.document.details", "Scene Document", "Scene document details.", 10, true),
  },
  {
    traitKind: "HasEntities",
    summary: section("scene.entities.summary", "Entities", "Scene entities summary.", 20, true),
  },
  {
    traitKind: "HasScripts",
    summary: section("scripts.summary", "Scripts", "Script references summary.", 30, false),
    details: section("scripts.details", "Scripts", "Script reference details.", 30, true),
  },
  {
    traitKind: "HasDiagnostics",
    summary: section("diagnostics.summary", "Diagnostics", "Diagnostics summary.", 900, false),
    details: section("diagnostics.details", "Diagnostics", "Diagnostic details.", 900, true),
  },
  {
    traitKind: "HasIdentity",
    summary: section("identity.summary", "Identity", "Identity summary.", 10, true),
    details: section("identity.details", "Identity", "Identity fields.", 10, true),
  },
  {
    traitKind: "HasVisibility",
    summary: section("visibility.summary", "Visibility", "Visibility summary.", 20, true),
    details: section("visibility.details", "Visibility", "Visibility fields.", 20, true),
  },
  {
    traitKind: "HasComponents",
    summary: section("components.summary", "Components", "Component list summary.", 30, true),
    details: section("components.details", "Components", "Component details.", 30, true),
  },
  {
    traitKind: "Transformable2D",
    summary: section("transform2.summary", "Transform2", "2D transform summary.", 100, true),
    details: section("transform2.details", "Transform2", "2D transform fields.", 100, true),
    viewport: section("transform2.viewport", "Transform2 Gizmo", "2D transform viewport controls.", 100, false),
  },
  {
    traitKind: "UsesTransform2D",
    summary: section("transform2.owner.summary", "Owner Transform2", "Owner transform link.", 105, false),
  },
  {
    traitKind: "Renderable2D",
    summary: section("render2d.summary", "Render2D", "2D render summary.", 200, true),
    details: section("render2d.details", "Render2D", "2D render fields.", 200, true),
  },
  {
    traitKind: "HasBounds2D",
    summary: section("bounds2.summary", "Bounds2D", "2D bounds summary.", 300, true),
    details: section("bounds2.details", "Bounds2D", "2D bounds fields.", 300, true),
    viewport: section("bounds2.viewport", "Bounds2D Overlay", "2D bounds viewport overlay.", 300, false),
  },
  {
    traitKind: "HasAssetRefs",
    summary: section("assetRefs.summary", "Asset References", "Asset reference summary.", 400, true),
    details: section("assetRefs.details", "Asset References", "Asset reference details.", 400, true),
  },
  {
    traitKind: "Selectable",
    summary: section("selection.summary", "Selection", "Selection metadata.", 450, false),
  },
  {
    traitKind: "Collidable2D",
    summary: section("collision.summary", "Collision", "Collision summary.", 500, true),
    details: section("collision.details", "Collision", "Collision fields.", 500, true),
    viewport: section("collision.viewport", "Collision Overlay", "Collider viewport overlay.", 500, false),
  },
  {
    traitKind: "Trigger2D",
    summary: section("trigger.summary", "Trigger", "Trigger summary.", 510, true),
    details: section("trigger.details", "Trigger", "Trigger fields.", 510, true),
  },
  {
    traitKind: "Scriptable",
    summary: section("script.summary", "Script", "Script/behavior summary.", 600, true),
    details: section("script.details", "Script", "Script/behavior fields.", 600, true),
  },
  {
    traitKind: "EventSource",
    summary: section("events.source.summary", "Events", "Emitted events summary.", 620, false),
    details: section("events.source.details", "Events", "Emitted event fields.", 620, true),
  },
  {
    traitKind: "EventListener",
    summary: section("events.listener.summary", "Listeners", "Event listener summary.", 630, false),
    details: section("events.listener.details", "Listeners", "Event listener fields.", 630, true),
  },
  {
    traitKind: "SceneTransitionSource",
    summary: section("sceneTransition.summary", "Scene Transition", "Scene transition summary.", 640, false),
    details: section("sceneTransition.details", "Scene Transition", "Scene transition fields.", 640, true),
  },
  {
    traitKind: "InputBindable",
    summary: section("input.summary", "Input", "Input binding summary.", 650, true),
    details: section("input.details", "Input", "Input binding fields.", 650, true),
  },
  {
    traitKind: "UiEditable",
    summary: section("ui.summary", "UI", "UI metadata summary.", 700, true),
    details: section("ui.details", "UI", "UI metadata fields.", 700, true),
  },
  {
    traitKind: "HasUiTree",
    summary: section("uiTree.summary", "UI Tree", "UI tree summary.", 710, true),
    details: section("uiTree.details", "UI Tree", "UI tree fields.", 710, true),
  },
  {
    traitKind: "DataBindable",
    summary: section("dataBinding.summary", "Data Binding", "Data binding summary.", 720, false),
    details: section("dataBinding.details", "Data Binding", "Data binding fields.", 720, true),
  },
  {
    traitKind: "Camera",
    summary: section("camera.summary", "Camera", "Camera summary.", 250, true),
    details: section("camera.details", "Camera", "Camera fields.", 250, true),
    viewport: section("camera.viewport", "Camera Viewport", "Camera viewport controls.", 250, false),
  },
  {
    traitKind: "Motion2D",
    summary: section("motion2.summary", "Motion2D", "2D motion summary.", 760, true),
    details: section("motion2.details", "Motion2D", "2D motion fields.", 760, true),
  },
  {
    traitKind: "Poolable",
    details: section("pool.details", "Pool", "Pooling fields.", 820, true),
  },
  {
    traitKind: "LifetimeLimited",
    details: section("lifetime.details", "Lifetime", "Lifetime fields.", 815, true),
  },
  {
    traitKind: "GenericEditable",
    details: section("genericEditable.details", "Properties", "Generic metadata properties.", 900, true),
  },
  {
    traitKind: "Patchable",
    details: section("patchOps.details", "Patch Operations", "Available patch operations.", 950, false),
    contextMenu: section("patchOps.contextMenu", "Patch Actions", "Patch actions.", 950, false),
  },
  {
    traitKind: "HasEditorControls",
    details: section("editorControls.details", "Editor Controls", "Available editor controls.", 960, false),
  },
  {
    traitKind: "DiagnosticSource",
    summary: section("diagnosticSource.summary", "Diagnostics", "Target diagnostic summary.", 970, false),
    details: section("diagnosticSource.details", "Diagnostics", "Target diagnostics.", 970, false),
  },
];

const RULES_BY_TRAIT = new Map(
  TRAIT_SECTION_RULES.map((rule) => [rule.traitKind, rule]),
);

export function composeSectionsForResolvedTarget(
  target: ResolvedEditorTarget,
  metadata: EditorMetadataCatalogDto | null | undefined,
): EditorTargetComposedSection[] {
  return uniqueStrings(target.metadataTraits)
    .flatMap((traitKind) => sectionsForTrait(traitKind, metadata))
    .sort((left, right) => {
      return left.priority - right.priority || left.label.localeCompare(right.label);
    });
}

function sectionsForTrait(
  traitKind: string,
  metadata: EditorMetadataCatalogDto | null | undefined,
): EditorTargetComposedSection[] {
  const rule = RULES_BY_TRAIT.get(traitKind);
  const descriptor = findMetadataTraitDescriptor(metadata, traitKind);

  if (!rule) {
    return [
      {
        id: `${traitKind}.details`,
        label: descriptor?.label ?? traitKind,
        placement: "details",
        traitKind,
        description: descriptor?.description ?? "Metadata trait details.",
        priority: 1000,
        defaultExpanded: false,
      },
    ];
  }

  return (
    [
      rule.summary ? withPlacement(rule.summary, traitKind, "summary") : null,
      rule.details ? withPlacement(rule.details, traitKind, "details") : null,
      rule.viewport ? withPlacement(rule.viewport, traitKind, "viewport") : null,
      rule.contextMenu ? withPlacement(rule.contextMenu, traitKind, "contextMenu") : null,
    ].filter(Boolean) as EditorTargetComposedSection[]
  );
}

function section(
  id: string,
  label: string,
  description: string,
  priority: number,
  defaultExpanded: boolean,
): Omit<EditorTargetComposedSection, "traitKind" | "placement"> {
  return {
    id,
    label,
    description,
    priority,
    defaultExpanded,
  };
}

function withPlacement(
  section: Omit<EditorTargetComposedSection, "traitKind" | "placement">,
  traitKind: string,
  placement: EditorTargetSectionPlacement,
): EditorTargetComposedSection {
  return {
    ...section,
    traitKind,
    placement,
  };
}

function uniqueStrings(values: readonly string[]): string[] {
  return Array.from(new Set(values.filter(Boolean)));
}
