import { useEffect, useState } from "react";
import { AlertTriangle, Search, Trash2 } from "lucide-react";
import type { AssetRegistryDto, CreateAssetImportOptionsDto, EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto, EditorSceneSummaryDto, ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import { createAssetDescriptor, createProjectItem, deleteProjectFile, getAssetRegistry, pickProjectSourceFile } from "../../api/editorApi";
import { listenWindowBus } from "../../app/windowBus";
import { AssetTreePanel, type AssetDeleteTarget } from "../../assets/AssetTreePanel";
import { managedAssetFromProjectFile, projectFileFromRawAsset } from "../../assets/assetProjectFiles";
import { assetFolderVisualForKind, assetVisualForKind } from "../../assets/assetVisualRegistry";
import { AddItemDialog } from "../../add-item/AddItemDialog";
import type { AddItemDialogRequest, AddItemKind } from "../../add-item/addItemTypes";
import type { ComponentToolbarState, EditorComponentProps } from "../../editor-components/componentTypes";
import type { FolderViewGroup } from "../../ui/folder-view/FolderView";
import { FolderView } from "../../ui/folder-view/FolderView";
import type { FolderViewStatus } from "../../ui/folder-view/folderViewTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { resolveManagedAssetOpenRequest } from "../../main-window/workspaceOpenRouting";
import { AppDialog } from "../../ui/dialog/AppDialog";
import { flattenProjectFiles, normalizePath } from "../files/fileTreeSelectors";
import { isScriptFile } from "../scenes/sceneContextModel";
import { deriveAssetBrowserState, summarizeVisibleAssets } from "./assetBrowserModel";
import { resolveManagedAssetThumbnail, resolveRawAssetThumbnail } from "./assetThumbnailResolver";

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
  onRefreshProjectTree?: () => void;
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
    void refreshRegistry();
  }, [sessionId]);

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
        onRefreshProjectTree?.();
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
      onRefreshProjectTree?.();
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
      onRefreshProjectTree?.();
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
      setDeleteTarget(null);
      onProjectItemDeleted?.({ modId, path: target.relativePath });
      await refreshRegistry();
      onRefreshProjectTree?.();
    } catch (deleteError) {
      setError(deleteError instanceof Error ? deleteError.message : String(deleteError));
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
  const registryManaged = (registry?.managedAssets ?? []).filter(isMvpManagedAsset);
  const fallbackManaged = buildManagedAssetFallback(details.id, projectTree?.root);
  const managed = registryManaged.length
    ? registryManaged
    : fallbackManaged.filter((asset) => isMvpManagedAsset(asset));
  const raw = (registry?.rawFiles ?? []).filter(isMvpRawAsset);
  const viewMode = String(toolbarState?.viewMode ?? "tree");
  const kindFilter = String(toolbarState?.kind ?? "all");
  const issuesOnly = Boolean(toolbarState?.issuesOnly ?? false);
  const { filteredManaged, filteredRaw, groupedManaged, treeManaged } = deriveAssetBrowserState(managed, raw, {
    search,
    kindFilter,
    issuesOnly,
  });
  const treeRegistry: AssetRegistryDto = {
    sessionId,
    modId: details.id,
    rootPath: registry?.rootPath ?? details.rootPath,
    managedAssets: treeManaged,
    rawFiles: filteredRaw,
    diagnostics: registry?.diagnostics ?? [],
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
      {loading || busy ? <p className="muted workspace-note">Indexing assets...</p> : null}
      {error ? <p className="muted workspace-note">{error}</p> : null}
      {viewMode === "tree" ? (
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
      ) : viewMode === "tiles" ? (
        <AssetTileExplorer
          groupedManaged={groupedManaged}
          rawFiles={filteredRaw}
          selectedAssetKey={selectedAssetKey}
          selectedFilePath={selectedFilePath}
          onCreateDescriptor={createDescriptorFromRaw}
          onDeleteProjectFile={setDeleteTarget}
          onSelectAsset={selectManagedAsset}
          onSelectFile={onSelectFile}
        />
      ) : (
        <>
          <SectionTitle title={`Managed Assets ${filteredManaged.length ? `(${filteredManaged.length})` : ""}`} />
          {filteredManaged.length ? filteredManaged.slice(0, 120).map((asset) => renderManagedAssetRow(asset, selectedAssetKey, "list", onSelectAsset, setDeleteTarget)) : (
            <p className="muted workspace-note">No managed assets.</p>
          )}
        </>
      )}
      {viewMode === "list" && filteredRaw.length ? (
        <>
          <SectionTitle title={`Raw / Unmanaged (${filteredRaw.length})`} />
          {filteredRaw.slice(0, 120).map((file) => (
            <div key={file.relativePath} className={`workspace-row asset-registry-row ${selectedFilePath === file.relativePath ? "selected" : ""}`}>
              <button type="button" onClick={() => onSelectFile(projectFileFromRawAsset(file))}>
                <span className={`dock-icon asset-status-icon ${file.orphan ? "asset-status-warning" : "asset-status-valid"}`}>{rawAssetIcon(file.mediaType)}</span>
                <span>
                  <strong>{file.relativePath.split("/").pop()}</strong>
                  <small>{file.relativePath}</small>
                </span>
                <small className="asset-row-status">{file.orphan ? "orphan" : "referenced"}</small>
              </button>
              {file.orphan && file.mediaType.startsWith("image/") ? (
                <button type="button" className="workspace-row-action" onClick={() => void createDescriptorFromRaw(file)}>
                  descriptor
                </button>
              ) : null}
              <button
                type="button"
                className="workspace-row-action workspace-row-action-danger"
                title={`Delete ${file.relativePath}`}
                aria-label={`Delete ${file.relativePath}`}
                onClick={() => setDeleteTarget({ relativePath: file.relativePath, label: file.relativePath.split("/").pop() ?? file.relativePath, kind: "raw" })}
              >
                <Trash2 size={12} />
              </button>
            </div>
          ))}
        </>
      ) : null}
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

function renderManagedAssetRow(
  asset: ManagedAssetDto,
  selectedAssetKey: string | null,
  variant: "tree" | "list" = "list",
  onSelectAsset?: (asset: ManagedAssetDto) => void,
  onDeleteProjectFile?: (target: AssetDeleteTarget) => void,
) {
  return (
    <div key={asset.assetKey} className={`workspace-row asset-registry-row ${variant === "tree" ? "tree-row" : ""} ${selectedAssetKey === asset.assetKey ? "selected" : ""}`}>
      <button
        type="button"
        onClick={() => {
          onSelectAsset?.(asset);
        }}
      >
        <span className={`dock-icon asset-status-icon ${assetVisualForKind(asset.kind).tone} asset-status-${asset.status}`}>
          {assetIcon(asset.kind)}
        </span>
        <span>
          <strong>{asset.label}</strong>
          <small>{variant === "tree" ? asset.descriptorRelativePath : `${assetKindLabel(asset.kind)} · ${asset.assetKey}`}</small>
        </span>
        <small className="asset-row-status">{asset.status}</small>
      </button>
      {onDeleteProjectFile ? (
        <button
          type="button"
          className="workspace-row-action workspace-row-action-danger"
          title={`Delete ${asset.descriptorRelativePath}`}
          aria-label={`Delete ${asset.descriptorRelativePath}`}
          onClick={() => onDeleteProjectFile({
            relativePath: asset.descriptorRelativePath,
            label: asset.label,
            kind: "asset",
          })}
        >
          <Trash2 size={12} />
        </button>
      ) : null}
    </div>
  );
}

export function AssetRegistryTree({
  groupedManaged,
  rawFiles,
  selectedFilePath,
  onCreateDescriptor,
  onSelectFile,
}: {
  groupedManaged: globalThis.Map<string, ManagedAssetDto[]>;
  rawFiles: RawAssetFileDto[];
  selectedFilePath: string | null;
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const groups = Array.from(groupedManaged.entries());
  return (
    <div className="asset-registry-tree">
      <div className="asset-tree-root">
        <span className="tree-twist">▾</span>
        <span className={`dock-icon asset-status-icon ${assetFolderVisualForKind("root").tone}`}>{assetFolderIcon("root")}</span>
        <strong>Assets</strong>
        <small>{groups.reduce((count, [, assets]) => count + assets.length, 0)}</small>
      </div>
      {groups.map(([kind, assets]) => (
        <section key={kind} className="asset-tree-group">
          <div className="asset-tree-folder">
            <span className="tree-twist">▾</span>
            <span className={`dock-icon asset-status-icon ${assetFolderVisualForKind(kind).tone}`}>{assetFolderIcon(kind)}</span>
            <strong>{assetKindLabel(kind)}</strong>
            <small>{assets.length}</small>
          </div>
          <div className="asset-tree-children">
            {assets.map((asset) => renderManagedAssetRow(asset, null, "tree"))}
          </div>
        </section>
      ))}
      {rawFiles.length ? (
        <section className="asset-tree-group">
          <div className="asset-tree-folder">
            <span className="tree-twist">▾</span>
            <span className={`dock-icon asset-status-icon ${assetVisualForKind("image/raw").tone} asset-status-warning`}>{rawAssetIcon("image/raw")}</span>
            <strong>Raw Images</strong>
            <small>{rawFiles.length}</small>
          </div>
          <div className="asset-tree-children">
            {rawFiles.slice(0, 80).map((file) => renderRawAssetRow(file, selectedFilePath, onSelectFile, onCreateDescriptor))}
          </div>
        </section>
      ) : null}
    </div>
  );
}

export function AssetTileExplorer({
  groupedManaged,
  rawFiles,
  selectedAssetKey,
  selectedFilePath,
  onCreateDescriptor,
  onDeleteProjectFile,
  onSelectAsset,
  onSelectFile,
}: {
  groupedManaged: globalThis.Map<string, ManagedAssetDto[]>;
  rawFiles: RawAssetFileDto[];
  selectedAssetKey: string | null;
  selectedFilePath: string | null;
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>;
  onDeleteProjectFile?: (target: AssetDeleteTarget) => void;
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const groups: FolderViewGroup[] = [
    ...Array.from(groupedManaged.entries()).map(([kind, assets]) => ({
      id: kind,
      title: assetKindLabel(kind),
      subtitle: `${assets.length} managed assets`,
      icon: assetFolderIcon(kind),
      items: assets.map((asset) => {
        const visual = assetVisualForKind(asset.kind);
        const thumbnail = resolveManagedAssetThumbnail(asset);
        return {
          id: asset.assetKey,
          title: asset.label,
          subtitle: visual.label,
          thumbnailSrc: thumbnail.src,
          icon: visual.icon,
          status: folderStatusForAsset(asset.status),
          tone: visual.tone,
          selected: selectedAssetKey === asset.assetKey,
          kind: asset.kind,
          onOpen: () => {
            onSelectAsset?.(asset);
          },
          actions: onDeleteProjectFile ? [{
            id: "delete",
            label: <Trash2 size={11} />,
            title: "Delete asset",
            tone: "danger" as const,
            onRun: () => onDeleteProjectFile({
              relativePath: asset.descriptorRelativePath,
              label: asset.label,
              kind: "asset",
            }),
          }] : undefined,
        };
      }),
    })),
    {
      id: "raw-images",
      title: "Raw Images",
      subtitle: `${rawFiles.length} source files`,
      icon: rawAssetIcon("image/raw"),
      items: rawFiles.map((file) => {
        const thumbnail = resolveRawAssetThumbnail(file);
        return {
          id: file.relativePath,
          title: file.relativePath.split("/").pop() ?? file.relativePath,
          subtitle: file.orphan ? "Raw orphan" : "Raw referenced",
          thumbnailSrc: thumbnail.src,
          icon: rawAssetIcon(file.mediaType),
          status: file.orphan ? "warning" : "valid",
          tone: assetVisualForKind(file.mediaType).tone,
          selected: selectedFilePath === file.relativePath,
          kind: file.mediaType,
          onOpen: () => onSelectFile(projectFileFromRawAsset(file)),
          actions: [
            ...(file.orphan && file.mediaType.startsWith("image/") ? [{
            id: "descriptor",
            label: "descriptor",
            onRun: () => void onCreateDescriptor(file),
          }] : []),
            ...(onDeleteProjectFile ? [{
              id: "delete",
              label: <Trash2 size={11} />,
              title: "Delete raw source",
              tone: "danger" as const,
              onRun: () => onDeleteProjectFile({
                relativePath: file.relativePath,
                label: file.relativePath.split("/").pop() ?? file.relativePath,
                kind: "raw",
              }),
            }] : []),
          ],
        };
      }),
    },
  ];

  return (
    <FolderView
      density="compact"
      emptyMessage="No assets match the current filter."
      groups={groups}
      thumbnailMode="pixel"
    />
  );
}

function folderStatusForAsset(status: string): FolderViewStatus {
  if (status === "valid") return "valid";
  if (status === "missingSource") return "missing";
  if (status === "error") return "error";
  return "warning";
}

function renderRawAssetRow(
  file: RawAssetFileDto,
  selectedFilePath: string | null,
  onSelectFile: (file: EditorProjectFileDto) => void,
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>,
  onDeleteProjectFile?: (target: AssetDeleteTarget) => void,
) {
  return (
    <div key={file.relativePath} className={`workspace-row asset-registry-row tree-row ${selectedFilePath === file.relativePath ? "selected" : ""}`}>
      <button type="button" onClick={() => onSelectFile(projectFileFromRawAsset(file))}>
        <span className={`dock-icon asset-status-icon ${assetVisualForKind(file.mediaType).tone} ${file.orphan ? "asset-status-warning" : "asset-status-valid"}`}>
          {rawAssetIcon(file.mediaType)}
        </span>
        <span>
          <strong>{file.relativePath.split("/").pop()}</strong>
          <small>{file.relativePath}</small>
        </span>
        <small className="asset-row-status">{file.orphan ? "orphan" : "referenced"}</small>
      </button>
      {file.orphan && file.mediaType.startsWith("image/") ? (
        <button type="button" className="workspace-row-action" onClick={() => void onCreateDescriptor(file)}>
          descriptor
        </button>
      ) : null}
      {onDeleteProjectFile ? (
        <button
          type="button"
          className="workspace-row-action workspace-row-action-danger"
          title={`Delete ${file.relativePath}`}
          aria-label={`Delete ${file.relativePath}`}
          onClick={() => onDeleteProjectFile({
            relativePath: file.relativePath,
            label: file.relativePath.split("/").pop() ?? file.relativePath,
            kind: "raw",
          })}
        >
          <Trash2 size={12} />
        </button>
      ) : null}
    </div>
  );
}

function assetFolderIcon(kind: string) {
  return assetFolderVisualForKind(kind).icon;
}

function assetKindLabel(kind: string): string {
  return assetVisualForKind(kind).label;
}

function buildManagedAssetFallback(modId: string, root?: EditorProjectFileDto): ManagedAssetDto[] {
  if (!root) return [];
  return flattenProjectFiles(root)
    .filter((file) => ["audio", "font", "imageAsset", "sceneDocument", "sceneScript", "script", "scriptPackage", "tileset", "tilemap", "spritesheet"].includes(file.kind))
    .map((file) => managedAssetFromProjectFile(modId, file));
}

function assetIcon(kind: string) {
  return assetVisualForKind(kind).icon;
}

function rawAssetIcon(mediaType: string) {
  return assetVisualForKind(mediaType).icon;
}

function isMvpManagedAsset(asset: ManagedAssetDto): boolean {
  return ["audio", "font-2d", "image-2d", "scene", "prefab", "script", "ui-theme", "ui-document", "ui-main-menu", "ui-component", "tileset-2d", "tile-ruleset-2d", "tilemap-2d", "sprite-sheet-2d", "spritesheet-2d"].includes(asset.kind);
}

function isMvpRawAsset(file: RawAssetFileDto): boolean {
  return Boolean(file.relativePath);
}

function SectionTitle({ title }: { title: string }) {
  return <h3 className="workspace-section-title">{title}</h3>;
}
