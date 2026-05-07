import type { ReactNode } from "react";
import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, Boxes, FileCode2, ListTree, SearchCode } from "lucide-react";
import type {
  AssetRegistryDto,
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../api/dto";
import { getAssetRegistry } from "../../api/editorApi";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import {
  assetToTarget,
  projectFileToTarget,
  rawAssetToTarget,
  sceneEntityIdToTarget,
} from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { ShowYamlButton } from "../files/ShowYamlButton";
import { sceneYamlSource } from "../files/yamlSourceRefs";
import {
  sceneDiagnostics,
  sceneRelatedManagedAssets,
  sceneRelatedRawFiles,
  sceneRelatedScripts,
  type SceneContextTab,
} from "./sceneContextModel";

const TABS: Array<{ id: SceneContextTab; label: string }> = [
  { id: "scripts", label: "Scripts" },
  { id: "assets", label: "Assets" },
  { id: "entities", label: "Entities" },
  { id: "diagnostics", label: "Diagnostics" },
  { id: "source", label: "Source" },
];

export function SceneContextPanel({ context, services }: EditorComponentProps<WorkspaceRuntimeServices>) {
  const [activeTab, setActiveTab] = useState<SceneContextTab>("scripts");
  const [registry, setRegistry] = useState<AssetRegistryDto | null>(services.assetRegistry ?? null);
  const [registryError, setRegistryError] = useState<string | null>(null);

  const scene = services.selectedScene ?? null;
  const sessionId = context.sessionId ?? undefined;

  useEffect(() => {
    if (services.assetRegistry) {
      setRegistry(services.assetRegistry);
      return;
    }
    if (!sessionId || !scene) {
      setRegistry(null);
      setRegistryError(null);
      return;
    }

    let alive = true;
    void getAssetRegistry(sessionId)
      .then((next) => {
        if (!alive) return;
        setRegistry(next);
        setRegistryError(null);
      })
      .catch((error) => {
        if (alive) setRegistryError(error instanceof Error ? error.message : String(error));
      });

    return () => {
      alive = false;
    };
  }, [services.assetRegistry, sessionId, scene?.id]);

  const scripts = useMemo(() => sceneRelatedScripts(services.projectTree, scene), [services.projectTree, scene]);
  const managedAssets = useMemo(() => sceneRelatedManagedAssets(registry, scene), [registry, scene]);
  const rawFiles = useMemo(() => sceneRelatedRawFiles(registry, scene), [registry, scene]);
  const diagnostics = sceneDiagnostics(scene);

  if (!scene) {
    return <p className="muted workspace-empty">No active scene selected.</p>;
  }

  return (
    <div className="dock-scroll">
      <section className="workspace-section">
        <h3>{scene.label}</h3>
        <p className="muted workspace-note">{scene.documentPath}</p>
        <div className="scene-context-tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={`button button-tool ${activeTab === tab.id ? "selected" : ""}`}
              type="button"
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </section>

      {activeTab === "scripts" ? (
        <ScriptsTab
          files={scripts}
          onSelectFile={(file) => services.activateEditorTarget?.(projectFileToTarget(file), "open")}
        />
      ) : null}
      {activeTab === "assets" ? (
        <AssetsTab
          managedAssets={managedAssets}
          rawFiles={rawFiles}
          registryError={registryError}
          onSelectAsset={(asset) => services.activateEditorTarget?.(assetToTarget(asset), "select")}
          onSelectRawFile={(file) => services.activateEditorTarget?.(rawAssetToTarget(file), "select")}
        />
      ) : null}
      {activeTab === "entities" ? (
        <EntitiesTab
          entities={services.hierarchy?.entities ?? []}
          loading={services.hierarchyTask?.status === "running"}
          selectedEntityId={services.selectedEntity?.id ?? null}
          onSelectEntity={(entityId) => {
            services.activateEditorTarget?.(sceneEntityIdToTarget(scene.id, entityId), "select");
          }}
        />
      ) : null}
      {activeTab === "diagnostics" ? <DiagnosticsTab diagnostics={diagnostics} /> : null}
      {activeTab === "source" ? (
        <SourceTab
          scene={scene}
          onOpenScript={services.targetBridge?.openSceneScript}
          onShowYaml={services.targetBridge?.showYamlView}
        />
      ) : null}
    </div>
  );
}

function ScriptsTab({
  files,
  onSelectFile,
}: {
  files: EditorProjectFileDto[];
  onSelectFile?: (file: EditorProjectFileDto) => void;
}) {
  return (
    <section className="workspace-section">
      <SectionTitle icon={<SearchCode size={14} />} title={`Scripts ${files.length}`} />
      {files.length ? files.map((file) => (
        <button key={file.relativePath} className="workspace-row" type="button" onClick={() => onSelectFile?.(file)}>
          <span className="dock-icon dock-icon-green">Rh</span>
          <span>
            <strong>{file.name}</strong>
            <small>{file.relativePath}</small>
          </span>
          <em className="badge badge-muted">rhai</em>
        </button>
      )) : <p className="muted workspace-note">No scene scripts indexed.</p>}
    </section>
  );
}

function AssetsTab({
  managedAssets,
  rawFiles,
  registryError,
  onSelectAsset,
  onSelectRawFile,
}: {
  managedAssets: ManagedAssetDto[];
  rawFiles: RawAssetFileDto[];
  registryError: string | null;
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectRawFile?: (file: RawAssetFileDto) => void;
}) {
  return (
    <section className="workspace-section">
      <SectionTitle icon={<Boxes size={14} />} title={`Assets ${managedAssets.length + rawFiles.length}`} />
      {registryError ? <p className="muted workspace-note">{registryError}</p> : null}
      {managedAssets.map((asset) => (
        <button key={asset.assetKey} className="workspace-row" type="button" onClick={() => onSelectAsset?.(asset)}>
          <span className="dock-icon dock-icon-purple">{asset.kind.slice(0, 2).toUpperCase()}</span>
          <span>
            <strong>{asset.label}</strong>
            <small>{asset.assetKey}</small>
          </span>
          <em className="badge badge-muted">{asset.kind}</em>
        </button>
      ))}
      {rawFiles.map((file) => (
        <button
          key={file.relativePath}
          className="workspace-row"
          type="button"
          onClick={() => onSelectRawFile?.(file)}
        >
          <span className="dock-icon dock-icon-blue">Raw</span>
          <span>
            <strong>{file.relativePath.split("/").pop() ?? file.relativePath}</strong>
            <small>{file.relativePath}</small>
          </span>
          <em className="badge badge-muted">{file.mediaType}</em>
        </button>
      ))}
      {!managedAssets.length && !rawFiles.length ? (
        <p className="muted workspace-note">No referenced assets indexed for this scene.</p>
      ) : null}
    </section>
  );
}

function EntitiesTab({
  entities,
  loading,
  selectedEntityId,
  onSelectEntity,
}: {
  entities: EditorSceneEntityDto[];
  loading: boolean;
  selectedEntityId: string | null;
  onSelectEntity?: (entityId: string) => void;
}) {
  return (
    <section className="workspace-section">
      <SectionTitle icon={<ListTree size={14} />} title={`Entities ${entities.length}`} />
      {loading ? <p className="muted workspace-note">Indexing scene entities...</p> : null}
      {!loading && entities.length ? entities.map((entity) => (
        <button
          key={entity.id}
          type="button"
          className={`workspace-row ${entity.id === selectedEntityId ? "selected" : ""}`}
          onClick={() => onSelectEntity?.(entity.id)}
        >
          <span className="dock-icon dock-icon-blue">{entity.name.slice(0, 2).toUpperCase()}</span>
          <span>
            <strong>{entity.name}</strong>
            <small>
              {entity.componentCount} components
              {entity.tags.length ? ` · #${entity.tags.join(" #")}` : ""}
            </small>
          </span>
          <em className={`badge ${entity.visible ? "badge-valid" : "badge-muted"}`}>
            {entity.componentTypes[0] ?? "entity"}
          </em>
        </button>
      )) : null}
      {!loading && !entities.length ? (
        <p className="muted workspace-note">No authored entities found in this scene document.</p>
      ) : null}
    </section>
  );
}

function DiagnosticsTab({ diagnostics }: { diagnostics: EditorDiagnosticDto[] }) {
  return (
    <section className="workspace-section">
      <SectionTitle icon={<AlertTriangle size={14} />} title={`Diagnostics ${diagnostics.length}`} />
      {diagnostics.length ? diagnostics.map((diagnostic, index) => (
        <div key={`${diagnostic.code}:${index}`} className="workspace-row">
          <span className="dock-icon dock-icon-orange">!</span>
          <span>
            <strong>{diagnostic.code}</strong>
            <small>{diagnostic.message}</small>
          </span>
          <em className="badge badge-muted">{diagnostic.level}</em>
        </div>
      )) : <p className="muted workspace-note">No scene diagnostics.</p>}
    </section>
  );
}

function SourceTab({
  scene,
  onOpenScript,
  onShowYaml,
}: {
  scene: EditorSceneSummaryDto;
  onOpenScript?: NonNullable<WorkspaceRuntimeServices["targetBridge"]>["openSceneScript"];
  onShowYaml?: NonNullable<WorkspaceRuntimeServices["targetBridge"]>["showYamlView"];
}) {
  return (
    <section className="workspace-section">
      <SectionTitle icon={<FileCode2 size={14} />} title="Source" />
      <div className="scene-heading-actions">
        <ShowYamlButton source={sceneYamlSource(scene)} onShow={onShowYaml} />
        <button
          className="button button-tool"
          type="button"
          disabled={!scene.scriptPath || !onOpenScript}
          title={scene.scriptPath || "No script"}
          onClick={() => onOpenScript?.(scene)}
        >
          <FileCode2 size={14} />
          Open Script
        </button>
      </div>
    </section>
  );
}

function SectionTitle({ icon, title }: { icon: ReactNode; title: string }) {
  return (
    <div className="workspace-section-title">
      {icon}
      <span>{title}</span>
    </div>
  );
}
