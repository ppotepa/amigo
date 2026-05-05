import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function CachePanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const details = services.details ?? null;
  const preview = services.preview;
  return (
    <div className="dock-scroll">
      <section className="workspace-section">
        <h3>Preview Cache</h3>
        <dl className="kv-list">
          <dt>Project</dt>
          <dd>{details?.projectCacheId ?? "none"}</dd>
          <dt>Status</dt>
          <dd>{preview?.status ?? "missing"}</dd>
          <dt>Frames</dt>
          <dd>{preview?.frameCount ?? 0}</dd>
          <dt>Hash</dt>
          <dd>{preview?.sourceHash ?? "none"}</dd>
        </dl>
      </section>
    </div>
  );
}
