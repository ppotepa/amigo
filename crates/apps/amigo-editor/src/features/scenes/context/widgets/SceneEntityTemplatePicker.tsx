import { useMemo, useState } from "react";
import {
  SCENE_ENTITY_TEMPLATES,
  type SceneEntityTemplate,
} from "./sceneEntityTemplates";

export type SceneEntityTemplatePickerProps = {
  open: boolean;
  onClose: () => void;
  onPick: (template: SceneEntityTemplate) => void;
};

export function SceneEntityTemplatePicker({
  open,
  onClose,
  onPick,
}: SceneEntityTemplatePickerProps) {
  const [query, setQuery] = useState("");
  const templates = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) return SCENE_ENTITY_TEMPLATES;
    return SCENE_ENTITY_TEMPLATES.filter((template) =>
      [
        template.id,
        template.label,
        template.description,
        ...template.componentTypes,
        ...(template.requiresAssetKind ?? []),
      ].join(" ").toLowerCase().includes(normalizedQuery),
    );
  }, [query]);
  const groups = useMemo(() => {
    const map = new Map<string, SceneEntityTemplate[]>();
    for (const template of templates) {
      const bucket = map.get(template.category) ?? [];
      bucket.push(template);
      map.set(template.category, bucket);
    }
    return Array.from(map.entries());
  }, [templates]);

  if (!open) return null;

  return (
    <div className="context-picker" role="dialog" aria-label="Add scene entity">
      <header className="context-picker-header">
        <strong>Add Entity</strong>
        <button type="button" className="context-icon-button" title="Close" onClick={onClose}>
          x
        </button>
      </header>
      <input
        className="context-picker-search"
        type="search"
        value={query}
        placeholder="Filter templates"
        onChange={(event) => setQuery(event.currentTarget.value)}
      />
      <div className="context-picker-list">
        {groups.length ? groups.map(([category, groupedTemplates]) => (
          <section key={category} className="context-picker-group">
            <h4>{category}</h4>
            {groupedTemplates.map((template) => (
              <button
                key={template.id}
                type="button"
                className="context-picker-item"
                onClick={() => onPick(template)}
              >
                <span>{template.label}</span>
                <small>{template.description}</small>
                {template.componentTypes.length ? (
                  <small>{template.componentTypes.join(" + ")}</small>
                ) : null}
                {template.requiresAssetKind?.length ? (
                  <small>Requires {template.requiresAssetKind.join(", ")}</small>
                ) : null}
                <small className="muted">{template.defaultYamlPreview}</small>
              </button>
            ))}
          </section>
        )) : (
          <p className="muted workspace-note">No matching templates.</p>
        )}
      </div>
    </div>
  );
}
