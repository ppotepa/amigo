import { WidgetFrame } from "./WidgetFrame";
import type { ChangesWidgetModel } from "./widgetTypes";

export function ChangesWidget({ model }: { model?: ChangesWidgetModel }) {
  if (!model) {
    return (
      <WidgetFrame id="changes-widget" title="Changes" compact>
        <p className="muted workspace-empty">Changes tracking not connected yet.</p>
      </WidgetFrame>
    );
  }

  return (
    <WidgetFrame id="changes-widget" title="Changes" compact>
      <dl className="workbench-dl">
        <div>
          <dt>Dirty</dt>
          <dd>{model.dirty ? "Yes" : "No"}</dd>
        </div>
        <div>
          <dt>Summary</dt>
          <dd>{model.summary ?? "No changes."}</dd>
        </div>
      </dl>
    </WidgetFrame>
  );
}
