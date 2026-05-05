import { Play } from "lucide-react";
import type { EditorProjectFileDto, EditorProjectTreeDto, EditorSceneSummaryDto } from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import { findProjectFile, normalizePath } from "../files/fileTreeSelectors";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function ScenesBrowserPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <ScenesBrowser
      details={services.details ?? null}
      onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
      onSelectScene={(scene) => services.selectScene?.(scene) ?? Promise.resolve()}
      projectTree={services.projectTree}
      selectedScene={services.selectedScene ?? null}
      toolbarState={services.toolbarState}
    />
  );
}

function ScenesBrowser({
  details,
  projectTree,
  selectedScene,
  toolbarState,
  onSelectScene,
  onSelectFile,
}: {
  details: { scenes: EditorSceneSummaryDto[] } | null;
  projectTree?: EditorProjectTreeDto;
  selectedScene: EditorSceneSummaryDto | null;
  toolbarState?: Record<string, string | boolean>;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  if (!details) {
    return <p className="muted workspace-empty">No scenes loaded.</p>;
  }

  const visibleOnly = Boolean(toolbarState?.visibleOnly ?? false);
  const viewMode = String(toolbarState?.viewMode ?? "list");
  const scenes = details.scenes
    .filter((scene) => !visibleOnly || scene.launcherVisible)
    .sort((left, right) => {
      if (viewMode !== "status") return 0;
      return sceneStatusRank(left.status) - sceneStatusRank(right.status) || left.label.localeCompare(right.label);
    });

  return (
    <div className="dock-scroll">
      <SectionTitle title={`Scenes ${scenes.length}`} />
      {scenes.map((scene) => {
        const document = projectTree ? findProjectFile(projectTree.root, relativeProjectPath(scene.documentPath)) : null;
        const script = projectTree ? findProjectFile(projectTree.root, relativeProjectPath(scene.scriptPath)) : null;
        return (
          <section key={scene.id} className={`workspace-section scene-browser-card ${scene.id === selectedScene?.id ? "selected" : ""}`}>
            <button type="button" className="workspace-row selected scene-browser-main" onClick={() => void onSelectScene(scene)}>
              <span className="dock-icon dock-icon-cyan"><Play size={13} /></span>
              <span>
                <strong>{scene.label}</strong>
                <small>{scene.id} · {scene.launcherVisible ? "launcher visible" : "hidden"}</small>
              </span>
              <em className={`badge status-${scene.status}`}>{scene.status}</em>
            </button>
            <div className="scene-browser-files">
              <button type="button" disabled={!document} onClick={() => document && onSelectFile(document)}>
                <span>scene.yml</span>
                <em className={`badge ${document ? "badge-valid" : "badge-warning"}`}>{document ? "yaml" : "missing"}</em>
              </button>
              <button type="button" disabled={!script} onClick={() => script && onSelectFile(script)}>
                <span>scene.rhai</span>
                <em className={`badge ${script ? "badge-valid" : "badge-warning"}`}>{script ? "rhai" : "missing"}</em>
              </button>
            </div>
          </section>
        );
      })}
      {scenes.length === 0 ? <p className="muted workspace-note">No scenes match the current toolbar filter.</p> : null}
    </div>
  );
}

function relativeProjectPath(path: string): string {
  const normalized = normalizePath(path);
  for (const prefix of ["scenes/", "raw/", "spritesheets/", "audio/", "fonts/", "scripts/", "data/", "docs/", "custom/", "packages/"]) {
    const index = normalized.indexOf(prefix);
    if (index >= 0) return normalized.slice(index);
  }
  return normalized;
}

function sceneStatusRank(status: string): number {
  if (status === "error" || status === "invalid") return 0;
  if (status === "warn" || status === "warning") return 1;
  if (status === "valid" || status === "ready" || status === "ok") return 2;
  return 3;
}

function SectionTitle({ title }: { title: string }) {
  return <div className="workspace-section-title">{title}</div>;
}
