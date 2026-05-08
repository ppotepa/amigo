import {
  AlertTriangle,
  Box,
  CheckCircle2,
  Folder,
  Gauge,
  Grid3X3,
  Image,
  Layers3,
  LayoutTemplate,
  List,
  ListTree,
  Package,
  Paintbrush,
  Plus,
  Play,
  RefreshCw,
  Settings,
  Terminal,
  Type,
} from "lucide-react";
import type React from "react";
import {
  ASSET_COMPONENTS,
  AssetsBrowserComponent,
  CacheManagerComponent,
  CachePreviewComponent,
  DIAGNOSTICS_COMPONENTS,
  DiagnosticsPanelComponent,
  DiagnosticsProblemsComponent,
  DocumentChangesComponent,
  EntityInspectorComponent,
  EntityPropertiesComponent,
  EventsLogComponent,
  FILE_COMPONENTS,
  FileAtlasComponent,
  FileBinaryComponent,
  FileConfigComponent,
  FileImageAssetComponent,
  FileManifestComponent,
  FilePackageComponent,
  FileRawImageComponent,
  FileSceneComponent,
  FileSceneScriptComponent,
  FileScriptComponent,
  FileSpriteComponent,
  FileTextComponent,
  FileTextureComponent,
  FileTileRulesetComponent,
  FileTilemapComponent,
  FileTilesetComponent,
  FilesBrowserComponent,
  INSPECTOR_COMPONENTS,
  PROJECT_COMPONENTS,
  ProjectCapabilitiesComponent,
  ProjectDependenciesComponent,
  ProjectExplorerComponent,
  ProjectOverviewComponent,
  SCENE_COMPONENTS,
  SYSTEM_COMPONENTS,
  SceneContextComponent,
  SceneHierarchyComponent,
  ScenePreviewComponent,
  ScenesBrowserComponent,
  ScriptsBrowserComponent,
  SettingsGlobalComponent,
  ScriptingConsoleComponent,
  TargetContextComponent,
  TasksMonitorComponent,
  ThemeControllerComponent,
  UI_DOCUMENT_COMPONENTS,
  UiDocumentEditorComponent,
  UiDocumentStructureComponent,
} from "./builtin";
import type { SemanticTone } from "../theme/semanticColorRegistry";
import type {
  ComponentContextRequirement,
  EditorComponentContext,
  EditorComponentDefinition,
  IconKey,
} from "./componentTypes";
import type { LucideIcon } from "lucide-react";

export {
  AssetsBrowserComponent,
  CacheManagerComponent,
  CachePreviewComponent,
  DiagnosticsPanelComponent,
  DiagnosticsProblemsComponent,
  DocumentChangesComponent,
  EntityInspectorComponent,
  EntityPropertiesComponent,
  EventsLogComponent,
  FileAtlasComponent,
  FileBinaryComponent,
  FileConfigComponent,
  FileImageAssetComponent,
  FileManifestComponent,
  FilePackageComponent,
  FileRawImageComponent,
  FileSceneComponent,
  FileSceneScriptComponent,
  FileScriptComponent,
  FileSpriteComponent,
  FileTextComponent,
  FileTextureComponent,
  FileTileRulesetComponent,
  FileTilemapComponent,
  FileTilesetComponent,
  FilesBrowserComponent,
  ProjectCapabilitiesComponent,
  ProjectDependenciesComponent,
  ProjectExplorerComponent,
  ProjectOverviewComponent,
  SceneContextComponent,
  SceneHierarchyComponent,
  ScenePreviewComponent,
  ScenesBrowserComponent,
  ScriptsBrowserComponent,
  SettingsGlobalComponent,
  ScriptingConsoleComponent,
  TargetContextComponent,
  TasksMonitorComponent,
  ThemeControllerComponent,
  UiDocumentEditorComponent,
  UiDocumentStructureComponent,
};

export const EDITOR_COMPONENTS = [
  ...PROJECT_COMPONENTS,
  ...ASSET_COMPONENTS,
  ...FILE_COMPONENTS,
  ...SCENE_COMPONENTS,
  ...INSPECTOR_COMPONENTS,
  ...DIAGNOSTICS_COMPONENTS,
  ...SYSTEM_COMPONENTS,
  ...UI_DOCUMENT_COMPONENTS,
] as const satisfies readonly EditorComponentDefinition<any>[];

const COMPONENTS_BY_ID = new Map<string, EditorComponentDefinition<any>>(
  EDITOR_COMPONENTS.map((component) => [component.id, component]),
);

export function editorComponentById(componentId: string): EditorComponentDefinition<any> | undefined {
  return COMPONENTS_BY_ID.get(componentId);
}

export function requireEditorComponentForBoundary(componentId: string): EditorComponentDefinition<any> {
  const component = editorComponentById(componentId);
  if (!component) {
    throw new Error(`Unknown editor component: ${componentId}`);
  }
  return component;
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

  const { capabilities: requiredCapabilities = [] } = component;
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
  "layout-template": LayoutTemplate,
  list: List,
  "list-tree": ListTree,
  package: Package,
  paintbrush: Paintbrush,
  plus: Plus,
  play: Play,
  refresh: RefreshCw,
  settings: Settings,
  terminal: Terminal,
  type: Type,
} satisfies Record<IconKey, LucideIcon>;

export function iconForEditorComponent(icon: IconKey, size = 14, tone?: SemanticTone): React.ReactNode {
  const Icon = EDITOR_COMPONENT_ICONS[icon] ?? Box;
  return <Icon size={size} className={`semantic-icon ${tone ?? "neutral"}`} aria-hidden="true" />;
}
