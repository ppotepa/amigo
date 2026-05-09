import { Check, Pencil, X } from "lucide-react";
import { useEffect, useState } from "react";
import { ContextActionStrip } from "../../../../ui/context-dock/ContextActionStrip";
import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import type { ContextAction } from "../../../../ui/context-dock/contextDockTypes";
import type { HeaderWidgetModel } from "../../../../workbench/widgets/HeaderWidget";
import { WidgetFrame } from "../../../../workbench/widgets/WidgetFrame";
import type { WidgetRenderProps } from "../../../../workbench/widgets/widgetTypes";
import { sceneContextIcon } from "../sceneContextIcons";
import type { SceneContextModel } from "../sceneContextTypes";

export function sceneHeaderToHeaderWidgetModel(
  model: SceneContextModel,
): HeaderWidgetModel {
  return {
    title: model.header.displayName,
    subtitle: model.scene.id,
    status: model.header.status,
    foldedHint: model.header.foldedHint,
    badges: model.header.badges,
  };
}

export function SceneHeaderWidget({
  model,
  services,
}: WidgetRenderProps<SceneContextModel>) {
  const [editing, setEditing] = useState(false);
  const [draftName, setDraftName] = useState(model.header.displayName);

  useEffect(() => {
    setDraftName(model.header.displayName);
  }, [model.header.displayName]);

  function cancelRename() {
    setDraftName(model.header.displayName);
    setEditing(false);
  }

  async function saveRename() {
    const nextName = draftName.trim();
    if (!nextName || nextName === model.header.displayName) {
      cancelRename();
      return;
    }
    await services.requestRenameScene?.({
      sceneId: model.scene.id,
      displayName: nextName,
    });
    setEditing(false);
  }

  const actions: ContextAction[] = editing
    ? [
        { id: "rename-save", label: "Save", icon: <Check size={13} />, onClick: () => void saveRename() },
        { id: "rename-cancel", label: "Cancel", icon: <X size={13} />, onClick: cancelRename },
      ]
    : [
        {
          id: "edit-display-name",
          label: "Edit Name",
          icon: <Pencil size={13} />,
          disabled: !model.header.canRename,
          onClick: () => setEditing(true),
        },
      ];

  return (
    <WidgetFrame
      id="scene.header"
      title="Scene Header"
      status={model.header.status}
      foldedHint={model.header.foldedHint}
    >
      <ContextRow icon={sceneContextIcon("scene")} title={model.header.displayName} subtitle={model.scene.id} tone="cyan" />
      {editing ? (
        <label className="context-input-row">
          <span className="context-input-label">Display Name</span>
          <input
            className="context-input"
            value={draftName}
            onChange={(event) => setDraftName(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                void saveRename();
              }
              if (event.key === "Escape") {
                event.preventDefault();
                cancelRename();
              }
            }}
          />
        </label>
      ) : null}
      {model.header.badges.map((badge) => (
        <ContextRow
          key={badge.id}
          title={badge.label}
          subtitle={badge.id}
          badge={badge.tone === "ok" ? "valid" : badge.tone === "neutral" ? "muted" : badge.tone}
        />
      ))}
      <ContextActionStrip actions={actions} />
    </WidgetFrame>
  );
}
