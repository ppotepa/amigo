import type {
  ComponentToolbarState,
  ComponentToolbarValue,
  EditorComponentToolbarDefinition,
} from "./componentTypes";
import { iconForEditorComponent } from "./componentRegistry";

export function defaultToolbarState(toolbar?: EditorComponentToolbarDefinition): ComponentToolbarState {
  const state: ComponentToolbarState = {};
  for (const control of toolbar?.controls ?? []) {
    if (control.kind === "action") continue;
    state[control.id] = control.defaultValue;
  }
  return state;
}

export function ComponentToolbar({
  onAction,
  onChange,
  state,
  toolbar,
}: {
  onAction: (controlId: string) => void;
  onChange: (controlId: string, value: ComponentToolbarValue) => void;
  state: ComponentToolbarState;
  toolbar: EditorComponentToolbarDefinition;
}) {
  return (
    <div className={`component-toolbar ${toolbar.compact ? "compact" : ""}`}>
      {toolbar.controls.map((control) => {
        if (control.kind === "segmented") {
          const value = String(state[control.id] ?? control.defaultValue);
          return (
            <div key={control.id} className="component-toolbar-segment" aria-label={control.label}>
              {control.options.map((option) => (
                <button
                  key={option.id}
                  type="button"
                  className={value === option.id ? "active" : ""}
                  title={option.label}
                  aria-label={option.label}
                  onClick={() => onChange(control.id, option.id)}
                >
                  {iconForEditorComponent(option.icon, 13)}
                </button>
              ))}
            </div>
          );
        }

        if (control.kind === "select") {
          const value = String(state[control.id] ?? control.defaultValue);
          return (
            <label key={control.id} className="component-toolbar-select">
              <span>{control.label}</span>
              <select value={value} onChange={(event) => onChange(control.id, event.target.value)}>
                {control.options.map((option) => (
                  <option key={option.id} value={option.id}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          );
        }

        if (control.kind === "toggle") {
          const checked = Boolean(state[control.id] ?? control.defaultValue);
          return (
            <button
              key={control.id}
              type="button"
              className={`component-toolbar-button ${checked ? "active" : ""}`}
              title={control.label}
              aria-label={control.label}
              aria-pressed={checked}
              onClick={() => onChange(control.id, !checked)}
            >
              {iconForEditorComponent(control.icon, 13)}
            </button>
          );
        }

        return (
          <button
            key={control.id}
            type="button"
            className="component-toolbar-button"
            title={control.label}
            aria-label={control.label}
            onClick={() => onAction(control.id)}
          >
            {iconForEditorComponent(control.icon, 13)}
          </button>
        );
      })}
    </div>
  );
}
