import type { ContextAction } from "./contextDockTypes";

export function ContextActionStrip({ actions }: { actions: ContextAction[] }) {
  return (
    <div className="context-action-strip">
      {actions.map((action) => (
        <button
          key={action.id}
          className="context-action-button"
          type="button"
          title={action.title ?? action.label}
          disabled={action.disabled}
          onClick={action.onClick}
        >
          {action.icon ? <span>{action.icon}</span> : null}
          <span>{action.label}</span>
        </button>
      ))}
    </div>
  );
}
