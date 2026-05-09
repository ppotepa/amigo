import type {
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../../api/dto";
import { sceneComponentIndexToTarget } from "../../../editor-targets/adapters/sceneTargetAdapter";
import { findProjectFile, flattenProjectFiles, normalizePath } from "../../files/fileTreeSelectors";
import { isScriptProjectFile, sceneScriptSource } from "../../files/scriptSourceRefs";
import { relativeProjectPath, sceneYamlSource } from "../../files/yamlSourceRefs";
// Legacy scene context implementation used by features/scene/target adapters.
// New target-facing code should import from features/scene/target/*.
import type {
  SceneChangesModel,
  SceneAssetGroup,
  SceneAssetGroupId,
  SceneComponentGroup,
  SceneComponentTreeItem,
  SceneComponentsModel,
  SceneEntitiesModel,
  SceneEntityNode,
  SceneHeaderModel,
  SceneNavigationLink,
  SceneNavigationModel,
  SceneScriptRef,
  SceneSourceModel,
} from "./sceneContextTypes";

const ASSET_GROUP_LABELS = {
  spritesheet: "Spritesheets",
  tilemap: "Tilemaps",
  audio: "Audio",
  font: "Fonts",
  scene: "Scenes",
  script: "Scripts",
  raw: "Raw Files",
  other: "Other",
} satisfies Record<SceneAssetGroupId, string>;

const ASSET_GROUP_ORDER: SceneAssetGroupId[] = [
  "spritesheet",
  "tilemap",
  "audio",
  "font",
  "raw",
  "script",
  "scene",
  "other",
];

export function selectSceneScripts(
  root: EditorProjectFileDto | undefined,
  scene: EditorSceneSummaryDto,
): SceneScriptRef[] {
  if (!root) return [];

  const allScripts = flattenProjectFiles(root).filter(isScriptProjectFile);
  const primary = selectSceneScriptFile(root, scene);
  const sceneDir = sceneDirectory(scene);

  const related = allScripts
    .map((file): SceneScriptRef | null => {
      const relative = normalizePath(file.relativePath);

      if (primary && file.relativePath === primary.relativePath) {
        return {
          id: file.relativePath,
          role: "primary",
          file,
        };
      }

      if (relative === "scripts/mod.rhai" || relative.endsWith("/scripts/mod.rhai")) {
        return {
          id: file.relativePath,
          role: "mod",
          file,
        };
      }

      if (relative.startsWith(`${sceneDir}/`) && relative.endsWith(".rhai")) {
        return {
          id: file.relativePath,
          role: "component",
          file,
        };
      }

      return null;
    })
    .filter((entry): entry is SceneScriptRef => Boolean(entry));

  return uniqueScripts(related).sort((left, right) =>
    scriptRoleRank(left.role) - scriptRoleRank(right.role) ||
    left.file.relativePath.localeCompare(right.file.relativePath),
  );
}

export function selectSceneScriptFile(
  root: EditorProjectFileDto | undefined,
  scene: EditorSceneSummaryDto,
): EditorProjectFileDto | null {
  if (!root || !scene.scriptPath) return null;
  const relativePath = relativeProjectPath(scene.scriptPath);
  return findProjectFile(root, relativePath);
}

export function selectSceneAssetGroups(
  scene: EditorSceneSummaryDto,
  managedAssets: ManagedAssetDto[],
  rawFiles: RawAssetFileDto[],
): SceneAssetGroup[] {
  const sceneAsset = managedAssets.find((asset) => isSceneAsset(asset, scene)) ?? null;
  const sceneReferenceKeys = sceneReferenceKeySet(scene);
  const referencedAssetKeys = new Set(sceneAsset?.references ?? []);

  const relatedManaged = managedAssets.filter((asset) => {
    if (sceneAsset && asset.assetKey === sceneAsset.assetKey) return false;
    if (referencedAssetKeys.has(asset.assetKey) || referencedAssetKeys.has(asset.assetId)) return true;
    return asset.usedBy.some((value) => sceneReferenceKeys.has(normalizePath(value)));
  });

  const relatedRaw = rawFiles.filter((file) =>
    file.referencedBy.some((value) => sceneReferenceKeys.has(normalizePath(value))),
  );

  const groups = new Map<string, SceneAssetGroup>();

  for (const asset of relatedManaged) {
    const id = asset.domain || "other";
    const group = ensureAssetGroup(groups, id);
    group.managedAssets.push(asset);
    group.count += 1;
  }

  for (const file of relatedRaw) {
    const id = rawGroupId(file);
    const group = ensureAssetGroup(groups, id);
    group.rawFiles.push(file);
    group.count += 1;
  }

  return Array.from(groups.values())
    .filter((group) => group.count > 0)
    .sort((left, right) => groupRank(left.id) - groupRank(right.id) || left.label.localeCompare(right.label));
}

export function selectSceneEntityNodes(
  entities: EditorSceneEntityDto[],
  selectedEntityId: string | null | undefined,
): SceneEntityNode[] {
  return entities.map((entity) => ({
    entity,
    selected: entity.id === selectedEntityId,
  }));
}

export function selectSceneDiagnostics(
  scene: EditorSceneSummaryDto,
  hierarchyDiagnostics: EditorDiagnosticDto[] = [],
): EditorDiagnosticDto[] {
  return [...scene.diagnostics, ...hierarchyDiagnostics];
}

export function selectSceneSourceModel(scene: EditorSceneSummaryDto): SceneSourceModel {
  return {
    yaml: sceneYamlSource(scene),
    script: sceneScriptSource(scene),
    folderPath: sceneDirectory(scene),
  };
}

export function selectSceneHeaderModel(
  scene: EditorSceneSummaryDto,
  diagnostics: EditorDiagnosticDto[],
  sceneChanges?: SceneChangesModel | null,
): SceneHeaderModel {
  const errorCount = diagnostics.filter((diagnostic) => diagnostic.level === "error").length;
  const warningCount = diagnostics.filter((diagnostic) => diagnostic.level === "warning").length;
  const badges = [];
  if (errorCount > 0) {
    badges.push({ id: "invalid", label: `${errorCount} errors`, tone: "error" as const });
  } else if (warningCount > 0) {
    badges.push({ id: "warnings", label: `${warningCount} warnings`, tone: "warning" as const });
  } else {
    badges.push({ id: "valid", label: "Valid", tone: "ok" as const });
  }
  badges.push({
    id: "kind",
    label: scene.launcherVisible ? "Launcher Scene" : "Background Scene",
    tone: "info" as const,
  });
  if (sceneChanges?.dirty) {
    badges.push({ id: "dirty", label: "Unsaved", tone: "warning" as const });
  }

  const status: SceneHeaderModel["status"] =
    errorCount > 0 ? "error" :
    warningCount > 0 ? "warning" :
    "ok";

  return {
    scene,
    status,
    displayName: scene.label || scene.id,
    canRename: true,
    badges,
    foldedHint: `${scene.label || scene.id} - ${scene.status}`,
  };
}

export function selectSceneNavigationModel(
  scene: EditorSceneSummaryDto,
  scripts: SceneScriptRef[],
  source: SceneSourceModel,
  entities: SceneEntityNode[],
): SceneNavigationModel {
  const entries = entities
    .filter((node) => isEntryEntity(node.entity))
    .map((node) => entityNavigationLink(node, "Entry"));
  const triggers = entities
    .filter((node) => entityText(node.entity).includes("trigger"))
    .map((node) => entityNavigationLink(node, "Trigger"));

  return {
    incoming: [],
    outgoing: [],
    entries,
    triggers,
    foldedHint: `${entries.length} entries, ${triggers.length} triggers`,
    entryScript: scripts.find((script) => script.role === "primary") ?? null,
    scripts,
    yaml: source.yaml,
  };
}

export function selectSceneComponentsModel(
  sceneId: string,
  entities: SceneEntityNode[],
  diagnostics: EditorDiagnosticDto[],
): SceneComponentsModel {
  const items = sceneComponentItems(sceneId, entities);
  const warningCount = diagnostics.filter((diagnostic) => diagnostic.level !== "info").length;
  const groups = groupComponentItems(items);
  return {
    groups,
    total: items.length,
    warningCount,
    foldedHint: `${items.length} components in ${groups.length} groups`,
  };
}

export function selectSceneEntitiesModel(
  entities: SceneEntityNode[],
  diagnostics: EditorDiagnosticDto[],
): SceneEntitiesModel {
  const groups = new Map<string, SceneEntityNode[]>();
  for (const node of entities) {
    const key = entityGroupId(node.entity);
    const bucket = groups.get(key) ?? [];
    bucket.push(node);
    groups.set(key, bucket);
  }

  return {
    entities,
    total: entities.length,
    visibleCount: entities.filter((node) => node.entity.visible).length,
    warningCount: diagnostics.filter((diagnostic) => diagnostic.level !== "info").length,
    groups: Array.from(groups.entries())
      .map(([id, values]) => ({
        id,
        label: labelForDomain(id),
        count: values.length,
        entityIds: values.map((node) => node.entity.id),
      }))
      .sort((left, right) => left.label.localeCompare(right.label)),
  };
}

export function groupComponentTypes(componentTypes: string[]): SceneComponentGroup[] {
  return groupComponentItems(componentTypes.map((componentType, index) => ({
    id: `legacy:${index}:${componentType}`,
    label: componentType,
    typeName: componentType,
    componentIndex: index,
    ownerKind: "scene" as const,
    summary: componentType,
    status: "neutral" as const,
    target: { kind: "component", sceneId: "", ownerKind: "scene", componentIndex: index, componentType },
  })));
}

function sceneComponentItems(sceneId: string, entities: SceneEntityNode[]): SceneComponentTreeItem[] {
  return entities.flatMap((node) => {
    const components = node.entity.components?.length
      ? node.entity.components.map((component) => ({
          componentIndex: component.componentIndex,
          typeName: component.typeName,
          label: component.label || component.typeName,
          summary: `${node.entity.name || node.entity.id} / ${component.yamlPath}`,
          status: component.diagnostics.some((diagnostic) => diagnostic.level === "error")
            ? "error" as const
            : component.diagnostics.some((diagnostic) => diagnostic.level === "warning")
              ? "warning" as const
              : "ok" as const,
        }))
      : node.entity.componentTypes.map((typeName, componentIndex) => ({
          componentIndex,
          typeName,
          label: typeName,
          summary: node.entity.name || node.entity.id,
          status: "neutral" as const,
        }));

    return components.map((component) => ({
      id: `${node.entity.id}:${component.componentIndex}:${component.typeName}`,
      label: component.label,
      typeName: component.typeName,
      componentIndex: component.componentIndex,
      ownerKind: "entity" as const,
      entityId: node.entity.id,
      summary: component.summary,
      status: component.status,
      target: sceneComponentIndexToTarget({
        sceneId,
        entityId: node.entity.id,
        componentIndex: component.componentIndex,
        componentType: component.typeName,
      }),
    }));
  });
}

function groupComponentItems(items: SceneComponentTreeItem[]): SceneComponentGroup[] {
  const groups = new Map<string, SceneComponentTreeItem[]>();
  for (const item of items) {
    const key = componentDomain(item.typeName);
    const bucket = groups.get(key) ?? [];
    bucket.push(item);
    groups.set(key, bucket);
  }

  return Array.from(groups.entries())
    .map(([id, values]) => ({
      id,
      label: labelForDomain(id),
      count: values.length,
      items: values.sort((left, right) => left.label.localeCompare(right.label)),
      status: values.some((item) => item.status === "error")
        ? "error" as const
        : values.some((item) => item.status === "warning")
          ? "warning" as const
          : "info" as const,
    }))
    .sort((left, right) => left.label.localeCompare(right.label));
}

function ensureAssetGroup(groups: Map<string, SceneAssetGroup>, id: string): SceneAssetGroup {
  const existing = groups.get(id);
  if (existing) return existing;

  const group: SceneAssetGroup = {
    id,
    label: isSceneAssetGroupId(id) ? ASSET_GROUP_LABELS[id] : titleCase(id),
    count: 0,
    managedAssets: [],
    rawFiles: [],
  };
  groups.set(id, group);
  return group;
}

function isSceneAsset(asset: ManagedAssetDto, scene: EditorSceneSummaryDto): boolean {
  const documentPath = relativeProjectPath(scene.documentPath);
  return asset.domain === "scene" && (
    normalizePath(asset.descriptorRelativePath) === documentPath ||
    normalizePath(asset.descriptorPath).endsWith(documentPath) ||
    asset.assetId === scene.id ||
    asset.assetKey.endsWith(`/scenes/${scene.id}`) ||
    asset.assetKey.endsWith(`/scenes/${scene.path}`)
  );
}

function sceneReferenceKeySet(scene: EditorSceneSummaryDto): Set<string> {
  return new Set([
    scene.id,
    scene.path,
    scene.label,
    relativeProjectPath(scene.path),
    relativeProjectPath(scene.documentPath),
    relativeProjectPath(scene.scriptPath),
  ].map((value) => normalizePath(value ?? "")).filter(Boolean));
}

function sceneDirectory(scene: EditorSceneSummaryDto): string {
  const documentPath = relativeProjectPath(scene.documentPath);
  return normalizePath(documentPath).replace(/\/scene\.ya?ml$/i, "");
}

function rawGroupId(file: RawAssetFileDto): string {
  if (file.mediaType.startsWith("image/")) return "spritesheet";
  if (file.mediaType.startsWith("audio/")) return "audio";
  return "raw";
}

function groupRank(id: string): number {
  const index = isSceneAssetGroupId(id) ? ASSET_GROUP_ORDER.indexOf(id) : -1;
  return index >= 0 ? index : ASSET_GROUP_ORDER.length;
}

function isSceneAssetGroupId(id: string): id is SceneAssetGroupId {
  return id in ASSET_GROUP_LABELS;
}

function scriptRoleRank(role: SceneScriptRef["role"]): number {
  return role === "primary" ? 0 : role === "component" ? 1 : role === "mod" ? 2 : 3;
}

function uniqueScripts(scripts: SceneScriptRef[]): SceneScriptRef[] {
  const map = new Map<string, SceneScriptRef>();
  for (const script of scripts) map.set(script.file.relativePath, script);
  return Array.from(map.values());
}

function titleCase(value: string): string {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}

function componentDomain(componentType: string): string {
  const value = componentType.toLowerCase();
  if (value.includes("sprite") || value.includes("render") || value.includes("text") || value.includes("shape")) return "rendering";
  if (value.includes("collider") || value.includes("physics") || value.includes("trigger") || value.includes("body")) return "physics";
  if (value.includes("motion") || value.includes("velocity") || value.includes("transform")) return "motion";
  if (value.includes("ui")) return "ui";
  if (value.includes("audio") || value.includes("sound")) return "audio";
  if (value.includes("script") || value.includes("behavior")) return "scripting";
  return "other";
}

function labelForDomain(domain: string): string {
  return domain
    .split("-")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function entityNavigationLink(node: SceneEntityNode, subtitle: string): SceneNavigationLink {
  return {
    id: node.entity.id,
    label: node.entity.name || node.entity.id,
    subtitle,
    targetEntityId: node.entity.id,
  };
}

function isEntryEntity(entity: EditorSceneEntityDto): boolean {
  const text = entityText(entity);
  return text.includes("entry") || text.includes("spawn") || text.includes("player");
}

function entityGroupId(entity: EditorSceneEntityDto): string {
  const text = entityText(entity);
  if (text.includes("player") || text.includes("spawn")) return "player-spawn";
  if (text.includes("enemy")) return "enemy";
  if (text.includes("trigger")) return "trigger";
  if (text.includes("camera")) return "camera";
  if (text.includes("ui")) return "ui";
  return "other";
}

function entityText(entity: EditorSceneEntityDto): string {
  return [
    entity.id,
    entity.name,
    ...(entity.tags ?? []),
    ...(entity.groups ?? []),
    ...(entity.componentTypes ?? []),
  ].join(" ").toLowerCase();
}
