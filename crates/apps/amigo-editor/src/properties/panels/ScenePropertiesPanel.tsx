import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { SceneSelection } from "../propertiesTypes";

export function ScenePropertiesPanel({ selection }: { selection: SceneSelection }) {
  const scene = selection.scene;
  return (
    <KeyValueSection
      title="Scene"
      rows={[
        { label: "ID", value: scene.id },
        { label: "Label", value: scene.label },
        { label: "Document", value: scene.documentPath, title: scene.documentPath },
        { label: "Script", value: scene.scriptPath ?? "none", title: scene.scriptPath },
        { label: "Launcher", value: scene.launcherVisible ? "visible" : "hidden" },
      ]}
    />
  );
}
