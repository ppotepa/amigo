import { invoke } from "@tauri-apps/api/core";
import {
  openModSettingsWindow as openModSettingsWindowClient,
  openSettingsWindow as openSettingsWindowClient,
  openThemeWindow as openThemeWindowClient,
  openWorkspaceWindow,
} from "./windowApi";
import { emitWorkspaceOpened } from "../app/windowBus";
import type {
  CacheInfoDto,
  CacheMaintenanceResultDto,
  CachePolicyDto,
  EditorModDetailsDto,
  EditorModSummaryDto,
  EditorProjectFileContentDto,
  EditorProjectTreeDto,
  EditorSceneHierarchyDto,
  EditorSessionDto,
  EditorSettingsDto,
  EditorWindowRegistryDto,
  OpenModResultDto,
  ScenePreviewDto,
  ThemeSettingsDto,
} from "./dto";

export async function listKnownMods(): Promise<EditorModSummaryDto[]> {
  return invoke("list_known_mods");
}

export async function getModDetails(modId: string): Promise<EditorModDetailsDto> {
  return invoke("get_mod_details", { modId });
}

export async function requestScenePreview(modId: string, sceneId: string, forceRegenerate = false): Promise<ScenePreviewDto> {
  return invoke("request_scene_preview", { modId, sceneId, forceRegenerate });
}

export async function openMod(modId: string, selectedSceneId?: string | null): Promise<OpenModResultDto> {
  return invoke("open_mod", { modId, selectedSceneId });
}

export async function openModWorkspace(modId: string, selectedSceneId?: string | null): Promise<OpenModResultDto> {
  const result = await openMod(modId, selectedSceneId);
  await openWorkspaceWindow(result.sessionId, result.modId);
  await emitWorkspaceOpened(result.sessionId, result.modId);
  return result;
}

export async function openThemeWindow(): Promise<void> {
  return openThemeWindowClient();
}

export async function openSettingsWindow(): Promise<void> {
  return openSettingsWindowClient();
}

export async function openModSettingsWindow(sessionId: string): Promise<void> {
  return openModSettingsWindowClient(sessionId);
}

export async function registerEditorWindow(label: string, kind: string, sessionId?: string | null): Promise<void> {
  return invoke("register_editor_window", { label, kind, sessionId });
}

export async function markEditorWindowFocused(label: string): Promise<void> {
  return invoke("mark_editor_window_focused", { label });
}

export async function unregisterEditorWindow(label: string): Promise<void> {
  return invoke("unregister_editor_window", { label });
}

export async function getWindowRegistry(): Promise<EditorWindowRegistryDto> {
  return invoke("get_window_registry");
}

export async function focusWorkspaceWindow(sessionId: string): Promise<void> {
  return invoke("focus_workspace_window", { sessionId });
}

export async function closeWorkspaceWindow(sessionId: string): Promise<void> {
  return invoke("close_workspace_window", { sessionId });
}

export async function getEditorSession(sessionId: string): Promise<EditorSessionDto> {
  return invoke("get_editor_session", { sessionId });
}

export async function closeEditorSession(sessionId: string): Promise<void> {
  return invoke("close_editor_session", { sessionId });
}

export async function validateMod(modId: string): Promise<EditorModDetailsDto> {
  return invoke("validate_mod", { modId });
}

export async function regenerateAllScenePreviews(modId: string): Promise<ScenePreviewDto[]> {
  return invoke("regenerate_all_scene_previews", { modId });
}

export async function revealModFolder(modId: string): Promise<string> {
  return invoke("reveal_mod_folder", { modId });
}

export async function revealSceneDocument(modId: string, sceneId: string): Promise<string> {
  return invoke("reveal_scene_document", { modId, sceneId });
}

export async function getSceneHierarchy(modId: string, sceneId: string): Promise<EditorSceneHierarchyDto> {
  return invoke("get_scene_hierarchy", { modId, sceneId });
}

export async function getProjectTree(modId: string): Promise<EditorProjectTreeDto> {
  return invoke("get_project_tree", { modId });
}

export async function readProjectFile(modId: string, relativePath: string): Promise<EditorProjectFileContentDto> {
  return invoke("read_project_file", { modId, relativePath });
}

export async function revealProjectFile(modId: string, relativePath: string): Promise<string> {
  return invoke("reveal_project_file", { modId, relativePath });
}

export async function getThemeSettings(): Promise<ThemeSettingsDto> {
  return invoke("get_theme_settings");
}

export async function setThemeSettings(themeId: string): Promise<ThemeSettingsDto> {
  return invoke("set_theme_settings", { themeId });
}

export async function setFontSettings(fontId: string): Promise<ThemeSettingsDto> {
  return invoke("set_font_settings", { fontId });
}

export async function getEditorSettings(): Promise<EditorSettingsDto> {
  return invoke("get_editor_settings");
}

export async function setEditorModsRoot(modsRoot: string): Promise<EditorSettingsDto> {
  return invoke("set_editor_mods_root", { modsRoot });
}

export async function resetEditorModsRoot(): Promise<EditorSettingsDto> {
  return invoke("reset_editor_mods_root");
}

export async function pickModsRoot(): Promise<string | null> {
  return invoke("pick_mods_root");
}

export async function getCacheInfo(): Promise<CacheInfoDto> {
  return invoke("get_cache_info");
}

export async function getCachePolicy(): Promise<CachePolicyDto> {
  return invoke("get_cache_policy");
}

export async function setCachePolicy(policy: CachePolicyDto): Promise<CachePolicyDto> {
  return invoke("set_cache_policy", { policy });
}

export async function runCacheMaintenance(): Promise<CacheMaintenanceResultDto> {
  return invoke("run_cache_maintenance");
}

export async function clearOrphanedProjectCaches(): Promise<CacheMaintenanceResultDto> {
  return invoke("clear_orphaned_project_caches");
}

export async function clearProjectCache(projectCacheId: string): Promise<void> {
  return invoke("clear_project_cache", { projectCacheId });
}

export async function clearPreviewCache(projectCacheId: string): Promise<void> {
  return invoke("clear_preview_cache", { projectCacheId });
}

export async function clearAllPreviewCache(): Promise<void> {
  return invoke("clear_all_preview_cache");
}

export async function revealCacheFolder(): Promise<string> {
  return invoke("reveal_cache_folder");
}
