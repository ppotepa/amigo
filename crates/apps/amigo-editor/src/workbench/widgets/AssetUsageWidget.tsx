import { WidgetFrame } from "./WidgetFrame";
import type { AssetUsageWidgetModel } from "./widgetTypes";

export function AssetUsageWidget({ model, onOpen }: {
  model: AssetUsageWidgetModel;
  onOpen?: (row: AssetUsageWidgetModel["items"][number]) => void;
}) {
  return (
    <WidgetFrame id="asset-usage-widget" title="Asset Usage" compact>
      {model.items.length === 0 ? (
        <p className="muted workspace-empty">No usage data.</p>
      ) : (
        <ul className="workbench-widget-list">
          {model.items.map((item) => (
            <li key={item.label}>
              <button
                type="button"
                className="workbench-widget-list-item"
                onClick={() => onOpen?.(item)}
              >
                <span>{item.label}</span>
                <small>{item.count}</small>
              </button>
            </li>
          ))}
        </ul>
      )}
    </WidgetFrame>
  );
}
