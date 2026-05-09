// Legacy scene-specific tab composition.
// TargetContract entrypoint lives in features/scene/target/sceneTargetView.tsx.
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import type { EditorTargetRef } from "../../../editor-targets";
import type { SceneContextModel } from "./sceneContextTypes";
import { SCENE_CONTEXT_WIDGETS } from "./widgets/sceneContextWidgets";

export function SceneContextTab({
  model,
  onSelectTarget,
  services,
}: {
  model: SceneContextModel;
  services: WorkspaceRuntimeServices;
  onSelectTarget?: (target: EditorTargetRef) => void;
}) {
  return (
    <div className="scene-context-tab">
      {SCENE_CONTEXT_WIDGETS
        .filter((widget) => widget.placement === "top")
        .sort((left, right) => left.order - right.order)
        .map((widget) => {
          const Widget = widget.render;
          return (
            <Widget
              key={widget.id}
              model={model}
              services={services}
              onSelectTarget={onSelectTarget}
            />
          );
        })}
    </div>
  );
}
