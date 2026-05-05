import type { SceneSelection } from "../propertiesTypes";

export function ScenePropertiesPanel({ selection }: { selection: SceneSelection }) {
  const scene = selection.scene;
  return (
    <section className="workspace-section">
      <h3>Scene</h3>
      <dl className="kv-list">
        <dt>ID</dt>
        <dd>{scene.id}</dd>
        <dt>Label</dt>
        <dd>{scene.label}</dd>
        <dt>Document</dt>
        <dd title={scene.documentPath}>{scene.documentPath}</dd>
        <dt>Script</dt>
        <dd title={scene.scriptPath}>{scene.scriptPath ?? "none"}</dd>
        <dt>Launcher</dt>
        <dd>{scene.launcherVisible ? "visible" : "hidden"}</dd>
      </dl>
    </section>
  );
}
