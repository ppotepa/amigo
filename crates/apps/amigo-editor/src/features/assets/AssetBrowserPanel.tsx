import { useEffect, useState } from "react";
import { AlertTriangle, Search } from "lucide-react";
import type { AssetRegistryDto, CreateAssetImportOptionsDto, EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto, EditorSceneSummaryDto, ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import { createAssetDescriptor, createProjectItem, deleteProjectFile, getAssetRegistry, pickProjectSourceFile } from "../../api/editorApi";
import { listenWindowBus } from "../../app/windowBus";
import { AssetTreePanel, type AssetDeleteTarget } from "../../assets/AssetTreePanel";
import { managedAssetFromProjectFile, projectFileFromRawAsset } from "../../assets/assetProjectFiles";
import { AddItemDialog } from "../../add-item/AddItemDialog";
import type { AddItemDialogRequest, AddItemKind } from "../../add-item/addItemTypes";
import type { ComponentToolbarState, EditorComponentProps } from "../../editor-components/componentTypes";
import {
  OperationNotice,
  type OperationNoticeValue,
} from "../../ui/feedback/OperationNotice";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { resolveManagedAssetOpenRequest } from "../../main-window/workspaceOpenRouting";
import { AppDialog } from "../../ui/dialog/AppDialog";
import { flattenProjectFiles, normalizePath } from "../files/fileTreeSelectors";
import { isScriptFile } from "../scenes/sceneContextModel";
import { deriveAssetBrowserState, summarizeVisibleAssets } from "./assetBrowserModel";

export function AssetBrowserPanel({ context, services }: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <AssetBrowser
      details={services.details ?? null}
      loading={services.projectTreeTask?.status === "running"}
      onRefreshProjectTree={services.onProjectTreeRefresh}
      onSelectAsset={(asset) => {
        const request = resolveManagedAssetOpenRequest({
          asset,
          details: services.details ?? null,
          projectTree: services.projectTree,
        });

        if (services.openWorkspaceEditor) {
          services.openWorkspaceEditor(request);
          return;
        }

        if (request.kind === "scene" && services.activateSceneContext) {
          void services.activateSceneContext(request.scene);
          return;
        }
        services.handleSelectAsset?.(asset);
      }}
      onSelectFile={(file) => {
        if (services.openProjectFileEditor) {
          services.openProjectFileEditor(file);
          return;
        }
        services.handleSelectProjectFile?.(file);
      }}
      projectTree={services.projectTree}
      selectedAssetKey={services.selectedAsset?.assetKey ?? null}
      selectedFilePath={services.selectedFile?.relativePath ?? null}
      sessionId={context.sessionId ?? undefined}
      toolbarState={services.toolbarState}
      onProjectItemCreated={(change) => services.recordEvent?.({ type: "ProjectItemCreated", ...change })}
      onProjectItemDeleted={(change) => services.recordEvent?.({ type: "ProjectItemDeleted", ...change })}
    />
  );
}

export function AssetBrowser({
  details,
  sessionId,
  projectTree,
  loading,
  selectedAssetKey,
  selectedFilePath,
  onSelectAsset,
  onSelectFile,
  onRefreshProjectTree,
  toolbarState,
  onProjectItemCreated,
  onProjectItemDeleted,
}: {
  details: EditorModDetailsDto | null;
  sessionId?: string;
  projectTree?: EditorProjectTreeDto;
  loading: boolean;
  selectedAssetKey: string | null;
  selectedFilePath: string | null;
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectFile: (file: EditorProjectFileDto) => void;
  onRefreshProjectTree?: () => void | Promise<void>;
  toolbarState?: ComponentToolbarState;
  onProjectItemCreated?: (change: {
    modId: string;
    itemKind: string;
    itemId: string;
    createdFiles: string[];
    updatedFiles: string[];
  }) => void;
  onProjectItemDeleted?: (change: { modId: string; path: string }) => void;
}) {
  const [registry, setRegistry] = useState<AssetRegistryDto | null>(null);
  const [search, setSearch] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [addItemRequest, setAddItemRequest] = useState<AddItemDialogRequest | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AssetDeleteTarget | null>(null);
  const [operationNotice, setOperationNotice] = useState<OperationNoticeValue | null>(null);

  async function refreshRegistry() {
    if (!sessionId) return;
    setBusy(true);
    setError(null);
    try {
      setRegistry(await getAssetRegistry(sessionId));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    // Session id may stay stable across mod switches, so clear stale registry
    // and force reload when the active mod changes.
    setRegistry(null);
    setError(null);
    void refreshRegistry();
  }, [sessionId, details?.id]);

  useEffect(() => {
    if (toolbarState?.refreshNonce) {
      void refreshRegistry();
    }
  }, [toolbarState?.refreshNonce]);

  useEffect(() => {
    if (toolbarState?.addNonce) {
      setAddItemRequest({ mode: "catalog", scope: { kind: "project-root" } });
    }
  }, [toolbarState?.addNonce]);

  useEffect(() => {
    if (!details?.id) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listenWindowBus((event) => {
      if (disposed) return;
      if (
        (event.type === "AssetRegistryChanged" || event.type === "AssetDescriptorChanged") &&
        event.payload.modId === details.id
      ) {
        void refreshRegistry();
        void Promise.resolve(onRefreshProjectTree?.());
      }
    }).then((cleanup) => {
      unlisten = cleanup;
      if (disposed) cleanup();
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [details?.id, sessionId]);

  async function createDescriptorFromRaw(raw: RawAssetFileDto) {
    setAddItemRequest({
      mode: "direct",
      scope: { kind: "asset-category", category: "spritesheets" },
      itemKind: "image",
      prefillRawFilePath: raw.relativePath,
    });
  }

  async function handleCreateProjectItem(payload: {
    kind: AddItemKind;
    itemId: string;
    label: string;
    targetFolder: string;
    createScript: boolean;
    launcherVisible: boolean;
    sourceFilePath?: string;
  }) {
    const modId = details?.id;
    if (!modId) return;

    setBusy(true);
    setError(null);
    try {
      const result = await createProjectItem(modId, {
        itemKind: payload.kind,
        itemId: payload.itemId,
        label: payload.label || null,
        targetFolder: payload.targetFolder || null,
        sourceFilePath: payload.sourceFilePath || null,
        options: {
          createScript: payload.createScript,
          launcherVisible: payload.launcherVisible,
        },
      });
      onProjectItemCreated?.({
        modId,
        itemKind: result.itemKind,
        itemId: result.itemId,
        createdFiles: result.createdFiles,
        updatedFiles: result.updatedFiles,
      });
      await refreshRegistry();
      await Promise.resolve(onRefreshProjectTree?.());
    } catch (itemError) {
      setError(itemError instanceof Error ? itemError.message : String(itemError));
      throw itemError;
    } finally {
      setBusy(false);
    }
  }

  async function handleCreateDescriptor(payload: {
    rawFilePath: string;
    kind: "image" | "sprite" | "tileset";
    assetId: string;
    importOptions?: CreateAssetImportOptionsDto | null;
  }) {
    if (!sessionId) return;
    setBusy(true);
    setError(null);
    try {
      const created = await createAssetDescriptor(sessionId, {
        rawFilePath: payload.rawFilePath,
        kind: payload.kind,
        assetId: payload.assetId,
        importOptions: payload.importOptions ?? null,
      });
      await refreshRegistry();
      await Promise.resolve(onRefreshProjectTree?.());
      onSelectAsset?.(created);
    } catch (itemError) {
      setError(itemError instanceof Error ? itemError.message : String(itemError));
      throw itemError;
    } finally {
      setBusy(false);
    }
  }

  async function handleDeleteTarget(target: AssetDeleteTarget) {
    const modId = details?.id;
    if (!modId) return;

    setBusy(true);
    setError(null);
    try {
      await deleteProjectFile(modId, target.relativePath);
      onProjectItemDeleted?.({ modId, path: target.relativePath });
      await refreshRegistry();
      await Promise.resolve(onRefreshProjectTree?.());
      setOperationNotice({
        tone: "success",
        message: `Deleted ${target.relativePath}.`,
        detail: "Project item changes are applied immediately.",
      });
      setDeleteTarget(null);
    } catch (deleteError) {
      const message = deleteError instanceof Error ? deleteError.message : String(deleteError);
      setError(message);
      setOperationNotice({
        tone: "error",
        message: `Failed to delete ${target.relativePath}.`,
        detail: message,
      });
      throw deleteError;
    } finally {
      setBusy(false);
    }
  }

  if (!details || !sessionId) {
    return <p className="muted workspace-empty">No assets loaded.</p>;
  }

  const scriptCount = projectTree
    ? flattenProjectFiles(projectTree.root).filter(isScriptFile).length
    : 0;
  const activeRegistry = registry?.modId === details.id ? registry : null;
  const registryManaged = (activeRegistry?.managedAssets ?? []).filter(isMvpManagedAsset);
  const fallbackManaged = buildManagedAssetFallback(details.id, projectTree?.root);
  const managed = registryManaged.length
    ? registryManaged
    : fallbackManaged.filter((asset) => isMvpManagedAsset(asset));
  const raw = (activeRegistry?.rawFiles ?? []).filter(isMvpRawAsset);
  const kindFilter = String(toolbarState?.kind ?? "all");
  const issuesOnly = Boolean(toolbarState?.issuesOnly ?? false);
  const { filteredManaged, filteredRaw, treeManaged } = deriveAssetBrowserState(managed, raw, {
    search,
    kindFilter,
    issuesOnly,
  });
  const treeRegistry: AssetRegistryDto = {
    sessionId,
    modId: details.id,
    rootPath: activeRegistry?.rootPath ?? details.rootPath,
    managedAssets: treeManaged,
    rawFiles: filteredRaw,
    diagnostics: activeRegistry?.diagnostics ?? [],
  };
  const selectManagedAsset = (asset: ManagedAssetDto) => {
    onSelectAsset?.(asset);
  };
  const projectEntries = projectTree ? flattenProjectFiles(projectTree.root) : [];
  const projectPaths = new Set(projectEntries.map((entry) => normalizePath(entry.relativePath)));
  const managedIdsByKind = new Map<string, Set<string>>();
  for (const asset of managed) {
    const key = normalizePath(asset.kind);
    const ids = managedIdsByKind.get(key) ?? new Set<string>();
    ids.add(normalizePath(asset.assetId));
    managedIdsByKind.set(key, ids);
  }

  const isItemIdTaken = ({
    kind,
    itemId,
    targetFolder,
    descriptorKind,
  }: {
    kind: AddItemKind;
    itemId: string;
    targetFolder: string;
    descriptorKind: "image" | "sprite" | "tileset";
  }): boolean => {
    const normalizedId = normalizePath(itemId);
    if (!normalizedId) return false;

    if (kind === "scene") {
      if (details.scenes.some((scene) => normalizePath(scene.id) === normalizedId)) {
        return true;
      }
      return projectPaths.has(normalizePath(`scenes/${normalizedId}/scene.yml`));
    }

    if (kind === "script") {
      return hasManagedId(managedIdsByKind, "script", normalizedId)
        || projectPaths.has(normalizePath(`scripts/${normalizedId}.rhai`));
    }

    if (kind === "font") {
      return hasManagedId(managedIdsByKind, "font-2d", normalizedId)
        || projectPaths.has(normalizePath(`fonts/${normalizedId}/font.yml`));
    }

    if (kind === "ui-theme") {
      return projectPaths.has(normalizePath(`ui/themes/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`ui/themes/${normalizedId}.yaml`))
        || projectPaths.has(normalizePath(`themes/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`themes/${normalizedId}.yaml`));
    }

    if (kind === "ui-document") {
      return projectPaths.has(normalizePath(`ui/documents/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`ui/documents/${normalizedId}.yaml`));
    }

    if (kind === "ui-main-menu") {
      return projectPaths.has(normalizePath(`ui/menus/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`ui/menus/${normalizedId}.yaml`));
    }

    if (kind === "ui-hud") {
      return projectPaths.has(normalizePath(`ui/hud/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`ui/hud/${normalizedId}.yaml`));
    }

    if (kind === "ui-dialog") {
      return projectPaths.has(normalizePath(`ui/dialogs/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`ui/dialogs/${normalizedId}.yaml`));
    }

    if (kind === "ui-component") {
      return projectPaths.has(normalizePath(`ui/components/${normalizedId}.yml`))
        || projectPaths.has(normalizePath(`ui/components/${normalizedId}.yaml`));
    }

    if (kind === "image") {
      if (descriptorKind === "image") {
        return hasManagedId(managedIdsByKind, "image-2d", normalizedId);
      }
      if (descriptorKind === "sprite") {
        return hasManagedId(managedIdsByKind, "sprite-sheet-2d", normalizedId)
          || hasManagedId(managedIdsByKind, "spritesheet-2d", normalizedId);
      }
      if (descriptorKind === "tileset") {
        return hasManagedId(managedIdsByKind, "tileset-2d", normalizedId);
      }
    }

    if (kind === "folder") {
      const folderPath = normalizePath([targetFolder, normalizedId].filter(Boolean).join("/"));
      if (!folderPath) return false;
      if (projectPaths.has(folderPath)) return true;
      for (const path of projectPaths) {
        if (path.startsWith(`${folderPath}/`)) return true;
      }
      return false;
    }

    return false;
  };

  return (
    <div className="dock-scroll">
      <div className="asset-browser-compact-header">
        <strong>Assets</strong>
        <span>{summarizeVisibleAssets(filteredManaged.length, filteredRaw.length, scriptCount)}</span>
      </div>
      <label className="workspace-search asset-browser-search">
        <Search size={13} />
        <input value={search} placeholder="Search assets..." onChange={(event) => setSearch(event.target.value)} />
      </label>
      <OperationNotice notice={operationNotice} onDismiss={() => setOperationNotice(null)} />
      {loading || busy ? <p className="muted workspace-note">Indexing assets...</p> : null}
      {error ? <p className="muted workspace-note">{error}</p> : null}
      <div className="asset-tree-view">
        <AssetTreePanel
          registry={treeRegistry}
          selectedAssetKey={selectedAssetKey}
          selectedFilePath={selectedFilePath}
          onCreateDescriptor={createDescriptorFromRaw}
          onDeleteProjectFile={setDeleteTarget}
          onAddItem={(request) => setAddItemRequest(request)}
          onSelectAsset={selectManagedAsset}
          onSelectRawFile={(file) => onSelectFile(projectFileFromRawAsset(file))}
        />
      </div>
      {addItemRequest ? (
        <AddItemDialog
          details={details}
          request={addItemRequest}
          onCancel={() => setAddItemRequest(null)}
          onCreateProjectItem={handleCreateProjectItem}
          onCreateDescriptor={handleCreateDescriptor}
          onPickSourceFile={pickProjectSourceFile}
          isItemIdTaken={isItemIdTaken}
        />
      ) : null}
      {deleteTarget ? (
        <AppDialog
          title="Delete asset"
          subtitle={deleteTarget.label}
          icon={<AlertTriangle className="semantic-icon action-danger" size={17} />}
          iconClassName="dialog-tone-danger"
          toneClassName="dialog-tone-danger"
          onClose={() => setDeleteTarget(null)}
          dialogClassName="confirm-dialog"
          bodyClassName="confirm-dialog-body"
          footer={(
            <>
              <button className="button button-ghost" type="button" disabled={busy} onClick={() => setDeleteTarget(null)}>
                Cancel
              </button>
              <button className="button button-danger" type="button" disabled={busy} onClick={() => void handleDeleteTarget(deleteTarget)}>
                {busy ? "Deleting..." : "Delete asset"}
              </button>
            </>
          )}
        >
          <p className="confirm-dialog-copy">
            This will delete <code>{deleteTarget.relativePath}</code> from the project. This cannot be undone.
          </p>
        </AppDialog>
      ) : null}
    </div>
  );
}

function hasManagedId(index: Map<string, Set<string>>, managedKind: string, itemId: string): boolean {
  const ids = index.get(normalizePath(managedKind));
  return ids?.has(normalizePath(itemId)) ?? false;
}

function buildManagedAssetFallback(modId: string, root?: EditorProjectFileDto): ManagedAssetDto[] {
  if (!root) return [];
  return flattenProjectFiles(root)
    .filter((file) => ["audio", "font", "imageAsset", "sceneDocument", "sceneScript", "script", "scriptPackage", "tileset", "tilemap", "spritesheet"].includes(file.kind))
    .map((file) => managedAssetFromProjectFile(modId, file));
}

function isMvpManagedAsset(asset: ManagedAssetDto): boolean {
  return ["audio", "font-2d", "image-2d", "scene", "prefab", "script", "ui-theme", "ui-document", "ui-main-menu", "ui-component", "tileset-2d", "tile-ruleset-2d", "tilemap-2d", "sprite-sheet-2d", "spritesheet-2d"].includes(asset.kind);
}

function isMvpRawAsset(file: RawAssetFileDto): boolean {
  return Boolean(file.relativePath);
}
