import { ShowYamlButton } from "../../features/files/ShowYamlButton";
import { sceneYamlSource } from "../../features/files/yamlSourceRefs";
import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { PropertiesContext, SceneSelection } from "../propertiesTypes";

export function ScenePropertiesPanel({
  context,
  selection,
}: {
  context: PropertiesContext;
  selection: SceneSelection;
}) {
  const scene = selection.scene;
  const source = sceneYamlSource(scene);
  return (
    <>
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

      {source ? (
        <section className="workspace-section">
          <h3>Source</h3>
          <ShowYamlButton source={source} onShow={context.onShowYaml} />
        </section>
      ) : null}
    </>
  );
}
