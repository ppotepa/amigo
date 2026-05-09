import { Plus } from "lucide-react";
import { ContextActionStrip } from "../../../../ui/context-dock/ContextActionStrip";
import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import { WidgetFrame } from "../../../../workbench/widgets/WidgetFrame";
import type { WidgetRenderProps } from "../../../../workbench/widgets/widgetTypes";
import { sceneContextIcon } from "../sceneContextIcons";
import type { SceneContextModel } from "../sceneContextTypes";

export function SceneNavigationWidget({
  model,
  onSelectTarget,
}: WidgetRenderProps<SceneContextModel>) {
  return (
    <WidgetFrame
      id="scene.navigation"
      title="Navigation"
      status="ok"
      foldedHint={model.navigation.foldedHint}
      defaultCollapsed={false}
    >
      <div className="scene-navigation-grid">
        <NavigationGroup title="Incoming" rows={model.navigation.incoming} />
        <NavigationGroup title="Outgoing" rows={model.navigation.outgoing} />
        <NavigationGroup
          title="Entry / Triggers"
          rows={[...model.navigation.entries, ...model.navigation.triggers]}
          onSelectTarget={onSelectTarget}
          sceneId={model.scene.id}
        />
      </div>
      <ContextActionStrip
        actions={[{
          id: "add-transition",
          label: "Add Transition",
          icon: <Plus size={13} />,
          disabled: true,
          onClick: () => undefined,
        }]}
      />
    </WidgetFrame>
  );
}

function NavigationGroup({
  onSelectTarget,
  rows,
  sceneId,
  title,
}: {
  title: string;
  rows: SceneContextModel["navigation"]["incoming"];
  sceneId?: string;
  onSelectTarget?: WidgetRenderProps<SceneContextModel>["onSelectTarget"];
}) {
  return (
    <div className="scene-navigation-group">
      <strong>{title}</strong>
      {rows.length ? rows.map((row) => (
        <ContextRow
          key={row.id}
          title={row.label}
          subtitle={row.subtitle}
          onClick={row.targetEntityId && sceneId
            ? () => onSelectTarget?.({ kind: "sceneEntity", sceneId, entityId: row.targetEntityId ?? "" })
            : undefined}
        />
      )) : (
        <p className="muted workspace-note">None</p>
      )}
    </div>
  );
}
