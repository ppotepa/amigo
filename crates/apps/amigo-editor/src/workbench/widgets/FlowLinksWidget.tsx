import { WidgetFrame } from "./WidgetFrame";
import type { FlowLinkRow, FlowLinksWidgetModel } from "./widgetTypes";

type FlowLinksWidgetProps = {
  model: FlowLinksWidgetModel;
  onAdd?: () => void;
  onSelectLink?: (item: FlowLinkRow) => void;
};

export function FlowLinksWidget({
  model,
  onAdd,
  onSelectLink,
}: FlowLinksWidgetProps) {
  return (
    <WidgetFrame
      id="flow-links-widget"
      title={model.title}
      status="neutral"
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
      <div className="workbench-flow-links">
        <section className="workbench-flow-links-column">
          <h4>Incoming</h4>
          <List rows={model.incoming} onSelect={onSelectLink} />
        </section>
        <section className="workbench-flow-links-column">
          <h4>Outgoing</h4>
          <List rows={model.outgoing} onSelect={onSelectLink} />
        </section>
        <section className="workbench-flow-links-column">
          <h4>Entry / Triggers</h4>
          <List rows={model.entries} onSelect={onSelectLink} />
          <List rows={model.triggers} onSelect={onSelectLink} />
        </section>
      </div>
    </WidgetFrame>
  );
}

function List({
  rows,
  onSelect,
}: {
  rows: FlowLinkRow[];
  onSelect?: (row: FlowLinkRow) => void;
}) {
  if (rows.length === 0) {
    return <p className="muted workspace-empty">No entries.</p>;
  }

  return (
    <ul className="workbench-widget-list">
      {rows.map((row) => (
        <li key={row.id}>
          <button type="button" className="workbench-widget-list-item" onClick={() => onSelect?.(row)}>
            <span className={`workbench-dot workbench-dot-${row.status ?? "neutral"}`} aria-hidden />
            <span>{row.label}</span>
            {row.subtitle ? <small>{row.subtitle}</small> : null}
          </button>
        </li>
      ))}
    </ul>
  );
}

