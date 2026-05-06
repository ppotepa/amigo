import { MousePointer2 } from "lucide-react";
import type { EditorUiNodeObjectDto } from "../../../api/dto";
import { ContextWidget } from "../../../ui/context-dock/ContextWidget";

export function UiNodeEditWidget({ node }: { node: EditorUiNodeObjectDto | null }) {
  if (!node) {
    return (
      <ContextWidget
        id="ui-node-edit"
        title="UI Node"
        icon={<MousePointer2 size={14} />}
        badge="none"
        badgeTone="muted"
        defaultCollapsed
      >
        <p className="muted workspace-note">Select a UI node to inspect menu controls.</p>
      </ContextWidget>
    );
  }

  return (
    <ContextWidget
      id="ui-node-edit"
      title="UI Node"
      icon={<MousePointer2 size={14} />}
      badge={node.nodeKind}
      badgeTone="info"
    >
      <div className="workspace-row">
        <span className="dock-icon dock-icon-cyan">{node.nodeKind.slice(0, 2).toUpperCase()}</span>
        <span>
          <strong>{node.label}</strong>
          <small>{node.nodePath}</small>
        </span>
      </div>

      <p className="muted workspace-note">
        Edit content and style in the Inspector. Visual drag/resize comes in a later step.
      </p>
    </ContextWidget>
  );
}
