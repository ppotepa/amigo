import { WidgetFrame } from "./WidgetFrame";
import type { DiagnosticsWidgetModel } from "./widgetTypes";

type DiagnosticsWidgetProps = {
  model?: DiagnosticsWidgetModel;
  onValidate?: () => void;
};

export function DiagnosticsWidget({ model, onValidate }: DiagnosticsWidgetProps) {
  const list = model?.diagnostics ?? [];

  return (
    <WidgetFrame id="diagnostics-widget" title="Diagnostics" compact>
      {list.length === 0 ? <p className="muted workspace-empty">No diagnostics.</p> : null}
      <ul className="workbench-widget-list">
        {list.map((diagnostic, index) => (
          <li key={diagnostic.id ?? index}>
            <div className="workbench-widget-list-item">
              <span className={`workbench-dot ${diagnostic.level === "error" ? "workbench-dot-error" : "workbench-dot-warning"}`} aria-hidden />
              <span>{diagnostic.message ?? "Unknown issue"}</span>
            </div>
          </li>
        ))}
      </ul>
      {onValidate ? (
        <div className="workbench-widget-actions-row">
          <button type="button" className="workbench-button" onClick={onValidate}>
            Validate
          </button>
        </div>
      ) : null}
    </WidgetFrame>
  );
}
