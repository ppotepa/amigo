import { useState } from "react";
import type { WidgetRenderProps } from "../../../../workbench/widgets/widgetTypes";
import { sceneEntityIdToTarget } from "../../../../editor-targets/adapters/sceneTargetAdapter";
import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import { WidgetFrame } from "../../../../workbench/widgets/WidgetFrame";
import { sceneContextIcon } from "../sceneContextIcons";
import type { SceneContextModel } from "../sceneContextTypes";
import { SceneEntityTemplatePicker } from "./SceneEntityTemplatePicker";

export function SceneEntitiesWidget({
  model,
  onSelectTarget,
  services,
}: WidgetRenderProps<SceneContextModel>) {
  const [pickerOpen, setPickerOpen] = useState(false);

  return (
    <WidgetFrame
      id="scene.entities"
      title="Entities"
      status={model.entitiesInfo.warningCount ? "warning" : "ok"}
      foldedHint={`${model.entitiesInfo.total} entities`}
      defaultCollapsed
      actions={(
        <button type="button" className="context-icon-button" title="Add entity" onClick={() => setPickerOpen(true)}>
          +
        </button>
      )}
    >
      <ContextRow title="Total" subtitle={`${model.entitiesInfo.total}`} />
      <ContextRow title="Visible" subtitle={`${model.entitiesInfo.visibleCount}`} />
      {model.entitiesInfo.groups.map((group) => (
        <ContextRow
          key={group.id}
          icon={sceneContextIcon("entities")}
          title={group.label}
          subtitle={group.entityIds.slice(0, 3).join(", ")}
          badge={group.count}
          onClick={() => {
            const entityId = group.entityIds[0];
            if (!entityId) return;
            const target = sceneEntityIdToTarget(model.scene.id, entityId);
            onSelectTarget?.(target);
            services.setCurrentDetailTarget?.(target);
            services.setActiveContextDetailsTab?.("properties");
          }}
        />
      ))}
      <ContextRow title="Add Entity" subtitle="Choose an entity template" onClick={() => setPickerOpen(true)} />
      <SceneEntityTemplatePicker
        open={pickerOpen}
        onClose={() => setPickerOpen(false)}
        onPick={(template) => {
          services.requestAddSceneEntity?.({
            sceneId: model.scene.id,
            templateId: template.id,
          });
          setPickerOpen(false);
        }}
      />
    </WidgetFrame>
  );
}
