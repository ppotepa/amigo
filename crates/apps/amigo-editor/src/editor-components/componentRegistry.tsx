import {
  AlertTriangle,
  Box,
  CheckCircle2,
  Folder,
  Gauge,
  Grid3X3,
  Image,
  Layers3,
  List,
  ListTree,
  Package,
  Paintbrush,
  Play,
  RefreshCw,
  Settings,
  Terminal,
  Type,
} from "lucide-react";
import type React from "react";
import { EDITOR_FEATURES } from "../features/editorFeatures";
import type {
  ComponentContextRequirement,
  EditorComponentContext,
  EditorComponentDefinition,
  IconKey,
} from "./componentTypes";
import type { LucideIcon } from "lucide-react";

export const EDITOR_COMPONENTS = EDITOR_FEATURES.flatMap(
  (feature) => feature.components ?? [],
);

const COMPONENTS_BY_ID = new Map(EDITOR_COMPONENTS.map((component) => [component.id, component]));

export function editorComponentById(componentId: string): EditorComponentDefinition | undefined {
  return COMPONENTS_BY_ID.get(componentId);
}

export function editorComponentsByCategory(category: string): EditorComponentDefinition[] {
  return EDITOR_COMPONENTS.filter((component) => component.category === category);
}

export function editorComponentsForPlacement(placementKind: string): EditorComponentDefinition[] {
  return EDITOR_COMPONENTS.filter((component) =>
    component.allowedPlacements.some((placement) => placement === placementKind),
  );
}

const REQUIREMENT_CHECKS = {
  editorSession: (context) => Boolean(context.sessionId),
  selectedMod: (context) => Boolean(context.modId),
  selectedScene: (context) => Boolean(context.selectedSceneId),
  selectedAsset: (context) => Boolean(context.selectedAssetId),
  selectedEntity: (context) => Boolean(context.selectedEntityId),
  projectCache: (context) => Boolean(context.modId),
  runtimePreview: (context) => Boolean(context.sessionId && context.selectedSceneId),
} satisfies Record<
  ComponentContextRequirement,
  (context: EditorComponentContext) => boolean
>;

export function canOpenEditorComponent(
  component: EditorComponentDefinition,
  context: EditorComponentContext,
): boolean {
  const requirements = component.requiredContext ?? [];
  const hasRequiredContext = requirements.every((requirement) =>
    REQUIREMENT_CHECKS[requirement](context),
  );

  if (!hasRequiredContext) return false;

  const requiredCapabilities = component.capabilities ?? [];
  if (requiredCapabilities.length === 0) return true;
  const capabilities = context.capabilities ?? [];
  return requiredCapabilities.every((capability) => capabilities.includes(capability));
}

const EDITOR_COMPONENT_ICONS = {
  "alert-triangle": AlertTriangle,
  box: Box,
  "check-circle": CheckCircle2,
  folder: Folder,
  gauge: Gauge,
  grid: Grid3X3,
  image: Image,
  layers: Layers3,
  list: List,
  "list-tree": ListTree,
  package: Package,
  paintbrush: Paintbrush,
  play: Play,
  refresh: RefreshCw,
  settings: Settings,
  terminal: Terminal,
  type: Type,
} satisfies Record<IconKey, LucideIcon>;

export function iconForEditorComponent(icon: IconKey, size = 14): React.ReactNode {
  const Icon = EDITOR_COMPONENT_ICONS[icon] ?? Box;
  return <Icon size={size} />;
}
