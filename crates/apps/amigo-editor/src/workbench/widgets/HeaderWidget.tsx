import type { ReactNode } from "react";
import { WidgetFrame } from "./WidgetFrame";
import type { HeaderWidgetModel } from "./widgetTypes";

export function HeaderWidget({
  actions,
  model,
}: {
  model: HeaderWidgetModel;
  actions?: ReactNode;
}) {
  return (
    <WidgetFrame
      id="target.header"
      title="Header"
      status={model.status}
      foldedHint={model.foldedHint}
      compact
      actions={actions}
    >
      <div className="workbench-header-widget">
        <strong>{model.title}</strong>
        {model.subtitle ? <small>{model.subtitle}</small> : null}
        {model.badges?.length ? (
          <div className="workbench-header-widget-badges">
            {model.badges.map((badge) => (
              <span
                key={badge.id}
                className={`workbench-badge workbench-badge-${badge.tone}`}
              >
                {badge.label}
              </span>
            ))}
          </div>
        ) : null}
      </div>
    </WidgetFrame>
  );
}
