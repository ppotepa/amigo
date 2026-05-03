import {
  AlertTriangle,
  Box,
  CheckCircle2,
  Folder,
  Gauge,
  Layers3,
  ListTree,
  Package,
  Paintbrush,
  Play,
  Settings,
  Terminal,
} from "lucide-react";
import type React from "react";
import { builtinEditorComponents } from "./builtinComponents";
import type {
  EditorComponentContext,
  EditorComponentDefinition,
  IconKey,
} from "./componentTypes";

export const EDITOR_COMPONENTS = builtinEditorComponents;

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

export function canOpenEditorComponent(
  component: EditorComponentDefinition,
  context: EditorComponentContext,
): boolean {
  const requirements = component.requiredContext ?? [];
  if (requirements.includes("editorSession") && !context.sessionId) return false;
  if (requirements.includes("selectedMod") && !context.modId) return false;
  if (requirements.includes("selectedScene") && !context.selectedSceneId) return false;
  if (requirements.includes("selectedEntity") && !context.selectedEntityId) return false;
  if (requirements.includes("selectedAsset") && !context.selectedAssetId) return false;
  if (requirements.includes("projectCache") && !context.modId) return false;

  const requiredCapabilities = component.capabilities ?? [];
  if (requiredCapabilities.length === 0) return true;
  const capabilities = context.capabilities ?? [];
  return requiredCapabilities.every((capability) => capabilities.includes(capability));
}

export function iconForEditorComponent(icon: IconKey, size = 14): React.ReactNode {
  switch (icon) {
    case "alert-triangle":
      return <AlertTriangle size={size} />;
    case "box":
      return <Box size={size} />;
    case "check-circle":
      return <CheckCircle2 size={size} />;
    case "folder":
      return <Folder size={size} />;
    case "gauge":
      return <Gauge size={size} />;
    case "layers":
      return <Layers3 size={size} />;
    case "list-tree":
      return <ListTree size={size} />;
    case "package":
      return <Package size={size} />;
    case "paintbrush":
      return <Paintbrush size={size} />;
    case "play":
      return <Play size={size} />;
    case "settings":
      return <Settings size={size} />;
    case "terminal":
      return <Terminal size={size} />;
    default:
      return <Box size={size} />;
  }
}
