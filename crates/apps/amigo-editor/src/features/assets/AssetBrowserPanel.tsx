import { useEffect, useState } from "react";
import { Search } from "lucide-react";
import type { AssetRegistryDto, CreateAssetImportOptionsDto, EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto, EditorSceneSummaryDto, ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import { createAssetDescriptor, getAssetRegistry } from "../../api/editorApi";
import { listenWindowBus } from "../../app/windowBus";
import { AssetTreePanel } from "../../assets/AssetTreePanel";
import { managedAssetFromProjectFile, projectFileFromRawAsset } from "../../assets/assetProjectFiles";
import { assetFolderVisualForKind, assetVisualForKind } from "../../assets/assetVisualRegistry";
import type { ComponentToolbarState, EditorComponentProps } from "../../editor-components/componentTypes";
import type { FolderViewGroup } from "../../ui/folder-view/FolderView";
import { FolderView } from "../../ui/folder-view/FolderView";
import type { FolderViewStatus } from "../../ui/folder-view/folderViewTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
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
        const scene = sceneForManagedAsset(services.details ?? null, asset);
        if (scene && services.activateSceneContext) {
          void services.activateSceneContext(scene);
          return;
        }
        services.handleSelectAsset?.(asset);
      }}
      onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
      projectTree={services.projectTree}
      selectedAssetKey={services.selectedAsset?.assetKey ?? null}
      selectedFilePath={services.selectedFile?.relativePath ?? null}
      sessionId={context.sessionId ?? undefined}
      toolbarState={services.toolbarState}
    />
  );
}

function sceneForManagedAsset(
  details: EditorModDetailsDto | null,
  asset: ManagedAssetDto,
): EditorSceneSummaryDto | null {
  if (!details || asset.kind !== "scene") return null;

  const descriptorPath = normalizePath(asset.descriptorRelativePath);
  const absoluteDescriptorPath = normalizePath(asset.descriptorPath);
  const assetKey = normalizePath(asset.assetKey);
  const assetId = normalizePath(asset.assetId);

  return details.scenes.find((scene) => {
    const sceneDocumentPath = normalizePath(scene.documentPath);
    const scenePath = normalizePath(scene.path);
    const sceneAssetKeys = [
      scenePath,
      sceneDocumentPath,
      `${scenePath}/scene.yml`,
      `${scenePath}/scene.yaml`,
      `scenes/${scene.id}`,
      `scenes/${scene.id}/scene.yml`,
      `scenes/${scene.id}/scene.yaml`,
    ].map(normalizePath);

    return sceneAssetKeys.some((key) => (
      descriptorPath === key ||
      absoluteDescriptorPath.endsWith(key) ||
      sceneDocumentPath.endsWith(descriptorPath) ||
      descriptorPath.endsWith(sceneDocumentPath) ||
      assetId === scene.id ||
      assetKey.endsWith(key)
    ));
  }) ?? null;
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
}) {
  const [registry, setRegistry] = useState<AssetRegistryDto | null>(null);
  const [search, setSearch] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
    if (!sessionId) return;
    const suggestedKind = suggestedDescriptorKind(raw);
    const kind = window.prompt("Descriptor kind: image, tileset, sprite", suggestedKind);
    if (!kind) return;
    const normalizedKind = kind.trim().toLowerCase();
    if (!["image", "tileset", "sprite"].includes(normalizedKind)) {
      setError("Only image, tileset and sprite descriptors are available in the current MVP.");
      return;
    }
    const suggestedId = raw.relativePath.split("/").pop()?.replace(/\.[^.]+$/, "").toLowerCase().replace(/[^a-z0-9-]+/g, "-") ?? "asset";
    const assetId = window.prompt("Asset id", suggestedId);
    if (!assetId) return;
    const importOptions = normalizedKind === "tileset" || normalizedKind === "sprite"
      ? promptImageSheetImportOptions(raw, normalizedKind)
      : null;
    if ((normalizedKind === "tileset" || normalizedKind === "sprite") && !importOptions) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const created = await createAssetDescriptor(sessionId, {
        rawFilePath: raw.relativePath,
        kind: normalizedKind,
        assetId,
        importOptions,
      });
      await refreshRegistry();
      onRefreshProjectTree?.();
      onSelectAsset?.(created);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
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

  return (
    <div className="dock-scroll">
      <label className="workspace-search">
        <Search size={13} />
        <input value={search} placeholder="Assets..." onChange={(event) => setSearch(event.target.value)} />
      </label>
      <p className="muted workspace-note">{summarizeVisibleAssets(filteredManaged.length, filteredRaw.length, scriptCount)}</p>
      {loading || busy ? <p className="muted workspace-note">Indexing assets...</p> : null}
      {error ? <p className="muted workspace-note">{error}</p> : null}
      {viewMode === "tree" ? (
        <div className="asset-tree-view">
          <AssetTreePanel
            registry={treeRegistry}
            selectedAssetKey={selectedAssetKey}
            selectedFilePath={selectedFilePath}
            onCreateDescriptor={createDescriptorFromRaw}
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
          onSelectAsset={selectManagedAsset}
          onSelectFile={onSelectFile}
        />
      ) : (
        <>
          <SectionTitle title={`Managed Assets ${filteredManaged.length ? `(${filteredManaged.length})` : ""}`} />
          {filteredManaged.length ? filteredManaged.slice(0, 120).map((asset) => renderManagedAssetRow(asset, selectedAssetKey, "list", onSelectAsset)) : (
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
            </div>
          ))}
        </>
      ) : null}
    </div>
  );
}

function renderManagedAssetRow(
  asset: ManagedAssetDto,
  selectedAssetKey: string | null,
  variant: "tree" | "list" = "list",
  onSelectAsset?: (asset: ManagedAssetDto) => void,
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
  onSelectAsset,
  onSelectFile,
}: {
  groupedManaged: globalThis.Map<string, ManagedAssetDto[]>;
  rawFiles: RawAssetFileDto[];
  selectedAssetKey: string | null;
  selectedFilePath: string | null;
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>;
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
          actions: file.orphan && file.mediaType.startsWith("image/") ? [{
            id: "descriptor",
            label: "descriptor",
            onRun: () => void onCreateDescriptor(file),
          }] : undefined,
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

function suggestedDescriptorKind(file: RawAssetFileDto): string {
  if (file.mediaType.startsWith("image/")) return "image";
  return "image";
}

function promptImageSheetImportOptions(
  file: RawAssetFileDto,
  kind: "tileset" | "sprite",
): CreateAssetImportOptionsDto | null {
  const imageWidth = Math.max(1, file.width ?? 0);
  const imageHeight = Math.max(1, file.height ?? 0);
  const tileWidth = promptPositiveInt(`${kind} tile width`, 32);
  if (tileWidth == null) return null;
  const tileHeight = promptPositiveInt(`${kind} tile height`, 32);
  if (tileHeight == null) return null;
  const defaultColumns = imageWidth > 0 ? Math.max(1, Math.floor(imageWidth / tileWidth)) : 1;
  const defaultRows = imageHeight > 0 ? Math.max(1, Math.floor(imageHeight / tileHeight)) : 1;
  const columns = promptPositiveInt(`${kind} columns`, defaultColumns);
  if (columns == null) return null;
  const rows = promptPositiveInt(`${kind} rows`, defaultRows);
  if (rows == null) return null;
  const tileCount = promptPositiveInt(`${kind} tile/frame count`, columns * rows);
  if (tileCount == null) return null;
  const marginX = promptNonNegativeInt(`${kind} margin X`, 0);
  if (marginX == null) return null;
  const marginY = promptNonNegativeInt(`${kind} margin Y`, 0);
  if (marginY == null) return null;
  const spacingX = promptNonNegativeInt(`${kind} spacing X`, 0);
  if (spacingX == null) return null;
  const spacingY = promptNonNegativeInt(`${kind} spacing Y`, 0);
  if (spacingY == null) return null;
  const fps = kind === "sprite" ? promptPositiveInt("sprite fps", 12) : null;
  if (kind === "sprite" && fps == null) return null;

  return {
    tileWidth,
    tileHeight,
    columns,
    rows,
    tileCount,
    marginX,
    marginY,
    spacingX,
    spacingY,
    fps,
  };
}

function promptPositiveInt(label: string, fallback: number): number | null {
  const value = window.prompt(label, String(fallback));
  if (value == null) return null;
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    window.alert(`${label} must be a positive integer.`);
    return null;
  }
  return parsed;
}

function promptNonNegativeInt(label: string, fallback: number): number | null {
  const value = window.prompt(label, String(fallback));
  if (value == null) return null;
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed < 0) {
    window.alert(`${label} must be a non-negative integer.`);
    return null;
  }
  return parsed;
}

function assetIcon(kind: string) {
  return assetVisualForKind(kind).icon;
}

function rawAssetIcon(mediaType: string) {
  return assetVisualForKind(mediaType).icon;
}

function isMvpManagedAsset(asset: ManagedAssetDto): boolean {
  return ["audio", "font-2d", "image-2d", "scene", "script", "tileset-2d", "tile-ruleset-2d", "tilemap-2d", "sprite-sheet-2d", "spritesheet-2d"].includes(asset.kind);
}

function isMvpRawAsset(file: RawAssetFileDto): boolean {
  return Boolean(file.relativePath);
}

function SectionTitle({ title }: { title: string }) {
  return <h3 className="workspace-section-title">{title}</h3>;
}
