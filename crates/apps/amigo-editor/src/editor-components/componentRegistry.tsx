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

export function requireEditorComponent(componentId: string): EditorComponentDefinition {
  const component = editorComponentById(componentId);
  if (!component) {
    throw new Error(`Unknown editor component: ${componentId}`);
  }
  return component;
}

export const ProjectExplorerComponent = requireEditorComponent("project.explorer");
export const ProjectOverviewComponent = requireEditorComponent("project.overview");
export const ProjectCapabilitiesComponent = requireEditorComponent("project.capabilities");
export const ProjectDependenciesComponent = requireEditorComponent("project.dependencies");
export const AssetsBrowserComponent = requireEditorComponent("assets.browser");
export const FilesBrowserComponent = requireEditorComponent("files.browser");
export const ScriptsBrowserComponent = requireEditorComponent("scripts.browser");
export const ScenesBrowserComponent = requireEditorComponent("scenes.browser");
export const SceneContextComponent = requireEditorComponent("scene.context");
export const SceneHierarchyComponent = requireEditorComponent("scene.hierarchy");
export const ScenePreviewComponent = requireEditorComponent("scene.preview");
export const EntityInspectorComponent = requireEditorComponent("entity.inspector");
export const EntityPropertiesComponent = requireEditorComponent("entity.properties");
export const TargetContextComponent = requireEditorComponent("target.context");
export const DocumentChangesComponent = requireEditorComponent("document.changes");
export const DiagnosticsProblemsComponent = requireEditorComponent("diagnostics.problems");
export const DiagnosticsPanelComponent = requireEditorComponent("diagnostics.panel");
export const EventsLogComponent = requireEditorComponent("events.log");
export const TasksMonitorComponent = requireEditorComponent("tasks.monitor");
export const CachePreviewComponent = requireEditorComponent("cache.preview");
export const ScriptingConsoleComponent = requireEditorComponent("scripting.console");
export const UiDocumentStructureComponent = requireEditorComponent("ui.document.structure");
export const UiDocumentEditorComponent = requireEditorComponent("ui.document.editor");
export const FileManifestComponent = requireEditorComponent("file.manifest");
export const FileSceneComponent = requireEditorComponent("file.scene");
export const FileSceneScriptComponent = requireEditorComponent("file.scene-script");
export const FilePackageComponent = requireEditorComponent("file.package");
export const FileScriptComponent = requireEditorComponent("file.script");
export const FileTextureComponent = requireEditorComponent("file.texture");
export const FileImageAssetComponent = requireEditorComponent("file.image-asset");
export const FileRawImageComponent = requireEditorComponent("file.raw-image");
export const FileSpriteComponent = requireEditorComponent("file.sprite");
export const FileAtlasComponent = requireEditorComponent("file.atlas");
export const FileTilesetComponent = requireEditorComponent("file.tileset");
export const FileTileRulesetComponent = requireEditorComponent("file.tile-ruleset");
export const FileTilemapComponent = requireEditorComponent("file.tilemap");
export const FileConfigComponent = requireEditorComponent("file.config");
export const FileTextComponent = requireEditorComponent("file.text");
export const FileBinaryComponent = requireEditorComponent("file.binary");

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
