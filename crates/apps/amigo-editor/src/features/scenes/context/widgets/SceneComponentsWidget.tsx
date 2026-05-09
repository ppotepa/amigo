import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import { WidgetFrame } from "../../../../workbench/widgets/WidgetFrame";
import type { WidgetRenderProps } from "../../../../workbench/widgets/widgetTypes";
import { SceneComponentPicker } from "./SceneComponentPicker";
import { sceneContextIcon } from "../sceneContextIcons";
import type { SceneComponentTreeItem, SceneContextModel } from "../sceneContextTypes";
import { useState } from "react";

export function SceneComponentsWidget({
  model,
  onSelectTarget,
  services,
}: WidgetRenderProps<SceneContextModel>) {
  const [pickerOpen, setPickerOpen] = useState(false);

  function selectComponent(item: SceneComponentTreeItem) {
    if (onSelectTarget) {
      onSelectTarget(item.target);
      return;
    }
    services.setCurrentDetailTarget?.(item.target);
    services.setActiveContextDetailsTab?.("properties");
  }

  return (
    <WidgetFrame
      id="scene.components"
      title="Components"
      status={model.components.warningCount ? "warning" : "ok"}
      foldedHint={model.components.foldedHint}
      actions={(
        <button type="button" className="context-icon-button" title="Add component" onClick={() => setPickerOpen(true)}>
          +
        </button>
      )}
    >
      {model.components.groups.length ? model.components.groups.map((group) => (
        <div key={group.id} className="scene-context-component-group">
          <ContextRow
            icon={sceneContextIcon("component")}
            title={group.label}
            subtitle={`${group.count} components`}
            badge={group.count}
            tone="purple"
          />
          {group.items.map((item) => (
            <ContextRow
              key={item.id}
              icon={sceneContextIcon("component")}
              title={item.label}
              subtitle={item.summary ?? item.typeName}
              badge={item.ownerKind}
              tone={item.status === "error" ? "red" : item.status === "warning" ? "orange" : "default"}
              onClick={() => selectComponent(item)}
            />
          ))}
        </div>
      )) : (
        <p className="muted workspace-note">No scene components indexed.</p>
      )}
      <SceneComponentPicker
        open={pickerOpen}
        components={services.metadataCatalog?.components ?? []}
        onClose={() => setPickerOpen(false)}
        onPick={(descriptor) => {
          services.requestAddSceneComponent?.({
            sceneId: model.scene.id,
            componentType: descriptor.typeName ?? descriptor.type_name ?? descriptor.kind,
          });
          setPickerOpen(false);
        }}
      />
    </WidgetFrame>
  );
}
