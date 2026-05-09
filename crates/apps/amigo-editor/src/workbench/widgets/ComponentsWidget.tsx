import { useMemo, useState, type ReactNode } from "react";
import { WidgetFrame } from "./WidgetFrame";
import type {
  ComponentsWidgetGroup,
  ComponentsWidgetItem,
  ComponentsWidgetModel,
  WidgetStatus,
} from "./widgetTypes";

type ComponentsWidgetProps = {
  model: ComponentsWidgetModel;
  onAdd?: () => void;
  onSelectItem?: (item: ComponentsWidgetItem) => void;
};

export function ComponentsWidget({ model, onAdd, onSelectItem }: ComponentsWidgetProps) {
  const [query, setQuery] = useState("");
  const [collapsedGroups, setCollapsedGroups] = useState<Record<string, boolean>>({});

  const groups = useMemo(() => {
    if (!query.trim()) return model.groups;

    const needle = query.toLowerCase();
    return model.groups
      .map((group) => {
        const filtered = group.items.filter((item) =>
          item.label.toLowerCase().includes(needle)
          || (item.subtitle ?? "").toLowerCase().includes(needle),
        );
        return { ...group, items: filtered, count: filtered.length };
      })
      .filter((group) => group.count > 0);
  }, [model.groups, query]);

  function toggleGroup(group: ComponentsWidgetGroup) {
    setCollapsedGroups((state) => ({
      ...state,
      [group.id]: !state[group.id],
    }));
  }

  return (
    <WidgetFrame
      id="components-widget"
      title={model.title}
      status={statusFromCount(model.warningCount, model.total)}
      compact
      foldedHint={model.foldedHint}
      actions={
        onAdd ? (
          <button type="button" className="workbench-widget-action" onClick={onAdd}>
            Add
          </button>
        ) : null
      }
    >
      <div className="workbench-search-row">
        <input
          className="workbench-input"
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
          placeholder="Search components..."
        />
      </div>
      {groups.length === 0 ? (
        <p className="muted workspace-empty">No matching components.</p>
      ) : (
        groups.map((group) => {
          const collapsed = collapsedGroups[group.id];
          return (
            <section key={group.id} className="workbench-widget-group">
              <button
                type="button"
                className="workbench-widget-group-header"
                onClick={() => toggleGroup(group)}
              >
                <span>{group.label}</span>
                <span className="muted">{group.count}</span>
              </button>
              {!collapsed ? (
                <ul className="workbench-widget-list">
                  {group.items.map((item) => (
                    <li key={item.id}>
                      <button
                        type="button"
                        className="workbench-widget-list-item"
                        onClick={() => onSelectItem?.(item)}
                      >
                        <span
                          className={`workbench-dot workbench-dot-${item.status ?? "neutral"}`}
                          aria-hidden
                        />
                        <span>{item.label}</span>
                        {item.subtitle ? <small>{item.subtitle}</small> : null}
                      </button>
                    </li>
                  ))}
                </ul>
              ) : null}
            </section>
          );
        })
      )}
    </WidgetFrame>
  );
}

function statusFromCount(warnings: number, total: number): WidgetStatus {
  if (total === 0) return "neutral";
  if (warnings > 0) return "warning";
  return "ok";
}

