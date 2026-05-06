import type { AddItemDefinition } from "./addItemCatalog";
import type { AddItemFormValue, AddItemKind } from "./addItemTypes";

export function AddItemFormHost({
  busy,
  idInvalid,
  idValidationError,
  kind,
  onBackToCatalog,
  onPickSourceFile,
  onValuesChange,
  outputPreview,
  selectedDefinition,
  tone,
  values,
}: {
  busy: boolean;
  idInvalid: boolean;
  idValidationError?: string;
  kind: AddItemKind;
  onBackToCatalog?: () => void;
  onPickSourceFile?: () => Promise<string | null>;
  onValuesChange: (next: AddItemFormValue | ((current: AddItemFormValue) => AddItemFormValue)) => void;
  outputPreview: string[];
  selectedDefinition?: AddItemDefinition;
  tone: string;
  values: AddItemFormValue;
}) {
  return (
    <div className="add-item-compact">
      {selectedDefinition ? (
        <div className={`add-item-selected-type add-item-tone ${tone}`}>
          <span className={`add-item-catalog-icon add-item-tone ${tone}`}>
            <selectedDefinition.icon className={`semantic-icon ${tone}`} size={16} />
          </span>
          <div>
            <strong>{selectedDefinition.label}</strong>
            <small>{selectedDefinition.description}</small>
          </div>
          {onBackToCatalog ? (
            <button className="button button-ghost add-item-change-type" type="button" disabled={busy} onClick={onBackToCatalog}>
              Change
            </button>
          ) : null}
        </div>
      ) : null}
      <div className="add-item-divider" />

      <div className="add-item-grid">
        <label className="add-item-row">
          <span className="add-item-label">Name</span>
          <div className="add-item-control">
            <input
              className={idInvalid ? "is-invalid" : ""}
              aria-invalid={idInvalid}
              value={values.itemId}
              onChange={(event) => onValuesChange((current) => ({ ...current, itemId: event.target.value }))}
            />
            {idInvalid ? <small>{idValidationError}</small> : null}
          </div>
        </label>

        <label className="add-item-row">
          <span className="add-item-label">Target</span>
          <div className="add-item-control">
            <input value={values.targetFolder} onChange={(event) => onValuesChange((current) => ({ ...current, targetFolder: event.target.value }))} />
          </div>
        </label>

        <label className="add-item-row">
          <span className="add-item-label">Label</span>
          <div className="add-item-control">
            <input value={values.label} onChange={(event) => onValuesChange((current) => ({ ...current, label: event.target.value }))} />
          </div>
        </label>

        {kind === "raw-source" || kind === "image" ? (
          <label className="add-item-row">
            <span className="add-item-label">Source</span>
            <div className="add-item-control add-item-control-inline">
              <input value={values.sourceFilePath} onChange={(event) => onValuesChange((current) => ({ ...current, sourceFilePath: event.target.value }))} />
              {onPickSourceFile ? (
                <button
                  type="button"
                  className="button button-ghost add-item-browse"
                  onClick={() => {
                    void onPickSourceFile().then((picked) => {
                      if (!picked) return;
                      onValuesChange((current) => ({ ...current, sourceFilePath: picked }));
                    });
                  }}
                >
                  Browse...
                </button>
              ) : null}
            </div>
          </label>
        ) : null}

        {kind === "image" ? (
          <label className="add-item-row">
            <span className="add-item-label">Kind</span>
            <div className="add-item-control">
              <select value={values.descriptorKind} onChange={(event) => onValuesChange((current) => ({ ...current, descriptorKind: event.target.value as "image" | "sprite" | "tileset" }))}>
                <option value="image">image</option>
                <option value="sprite">sprite</option>
                <option value="tileset">tileset</option>
              </select>
            </div>
          </label>
        ) : null}
      </div>

      {kind === "scene" ? (
        <div className="add-item-toggle-group">
          <label className="add-item-toggle-row">
            <input type="checkbox" checked={values.createScript} onChange={(event) => onValuesChange((current) => ({ ...current, createScript: event.target.checked }))} />
            <span>Create scene.rhai</span>
          </label>
          <label className="add-item-toggle-row">
            <input type="checkbox" checked={values.launcherVisible} onChange={(event) => onValuesChange((current) => ({ ...current, launcherVisible: event.target.checked }))} />
            <span>Visible in launcher</span>
          </label>
        </div>
      ) : null}

      <div className="add-item-divider" />

      <section className="add-item-output">
        <strong>Will create</strong>
        {outputPreview.length ? (
          <ul className="add-item-summary-list">
            {outputPreview.map((path) => (
              <li key={path}>
                <code>{path}</code>
              </li>
            ))}
          </ul>
        ) : (
          <p className="muted">Fill item id to preview output paths.</p>
        )}
      </section>
    </div>
  );
}
