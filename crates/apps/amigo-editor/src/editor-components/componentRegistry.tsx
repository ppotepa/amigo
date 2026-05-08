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
import { EDITOR_FEATURES } from "../features/editorFeatures";
import type { SemanticTone } from "../theme/semanticColorRegistry";
import type {
  ComponentContextRequirement,
  EditorComponentContext,
  EditorComponentDefinition,
  EditorComponentContextPayload,
  EditorComponentLaunchContext,
  FileWorkspaceComponentContext,
  IconKey,
  ScenePreviewComponentContext,
  UiDocumentEditorContext,
} from "./componentTypes";
import type { LucideIcon } from "lucide-react";

export const EDITOR_COMPONENTS = EDITOR_FEATURES.flatMap(
  (feature) => feature.components ?? [],
);

const COMPONENTS_BY_ID = new Map(EDITOR_COMPONENTS.map((component) => [component.id, component]));

export function editorComponentById(componentId: string): EditorComponentDefinition | undefined {
  return COMPONENTS_BY_ID.get(componentId);
}

export function requireEditorComponent<TContext extends EditorComponentContextPayload = EditorComponentContextPayload>(
  componentId: string,
): EditorComponentDefinition<TContext> {
  const component = editorComponentById(componentId);
  if (!component) {
    throw new Error(`Unknown editor component: ${componentId}`);
  }
  return component as unknown as EditorComponentDefinition<TContext>;
}

export const ProjectExplorerComponent = requireEditorComponent<EditorComponentLaunchContext>("project.explorer");
export const ProjectOverviewComponent = requireEditorComponent<EditorComponentLaunchContext>("project.overview");
export const ProjectCapabilitiesComponent = requireEditorComponent<EditorComponentLaunchContext>("project.capabilities");
export const ProjectDependenciesComponent = requireEditorComponent<EditorComponentLaunchContext>("project.dependencies");
export const AssetsBrowserComponent = requireEditorComponent<EditorComponentLaunchContext>("assets.browser");
export const FilesBrowserComponent = requireEditorComponent<EditorComponentLaunchContext>("files.browser");
export const ScriptsBrowserComponent = requireEditorComponent<EditorComponentLaunchContext>("scripts.browser");
export const ScenesBrowserComponent = requireEditorComponent<EditorComponentLaunchContext>("scenes.browser");
export const SceneContextComponent = requireEditorComponent<EditorComponentLaunchContext>("scene.context");
export const SceneHierarchyComponent = requireEditorComponent<EditorComponentLaunchContext>("scene.hierarchy");
export const ScenePreviewComponent = requireEditorComponent<ScenePreviewComponentContext>("scene.preview");
export const EntityInspectorComponent = requireEditorComponent<EditorComponentLaunchContext>("entity.inspector");
export const EntityPropertiesComponent = requireEditorComponent<EditorComponentLaunchContext>("entity.properties");
export const TargetContextComponent = requireEditorComponent<EditorComponentLaunchContext>("target.context");
export const DocumentChangesComponent = requireEditorComponent<EditorComponentLaunchContext>("document.changes");
export const DiagnosticsProblemsComponent = requireEditorComponent<EditorComponentLaunchContext>("diagnostics.problems");
export const DiagnosticsPanelComponent = requireEditorComponent<EditorComponentLaunchContext>("diagnostics.panel");
export const EventsLogComponent = requireEditorComponent<EditorComponentLaunchContext>("events.log");
export const TasksMonitorComponent = requireEditorComponent<EditorComponentLaunchContext>("tasks.monitor");
export const CachePreviewComponent = requireEditorComponent<EditorComponentLaunchContext>("cache.preview");
export const ScriptingConsoleComponent = requireEditorComponent<EditorComponentLaunchContext>("scripting.console");
export const UiDocumentStructureComponent = requireEditorComponent<EditorComponentLaunchContext>("ui.document.structure");
export const UiDocumentEditorComponent = requireEditorComponent<UiDocumentEditorContext>("ui.document.editor");
export const FileManifestComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.manifest");
export const FileSceneComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.scene");
export const FileSceneScriptComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.scene-script");
export const FilePackageComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.package");
export const FileScriptComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.script");
export const FileTextureComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.texture");
export const FileImageAssetComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.image-asset");
export const FileRawImageComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.raw-image");
export const FileSpriteComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.sprite");
export const FileAtlasComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.atlas");
export const FileTilesetComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.tileset");
export const FileTileRulesetComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.tile-ruleset");
export const FileTilemapComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.tilemap");
export const FileConfigComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.config");
export const FileTextComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.text");
export const FileBinaryComponent = requireEditorComponent<FileWorkspaceComponentContext>("file.binary");

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
