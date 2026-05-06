import type { AddItemDefinition } from "./addItemCatalog";
import type { AddItemKind } from "./addItemTypes";
import { createdPathsPreview, initialValues } from "./addItemDraft";
import { detailForAddItemKind, toneForAddItemKind } from "./addItemPresentation";

export function AddItemCatalogView({
  groups,
  onSelectKind,
  onConfigure,
  prefillRawFilePath,
  scope,
  selectedDefinition,
}: {
  groups: Array<{ category: string; label: string; entries: AddItemDefinition[] }>;
  onSelectKind: (kind: AddItemKind) => void;
  onConfigure: () => void;
  prefillRawFilePath?: string;
  scope: Parameters<typeof initialValues>[1];
  selectedDefinition?: AddItemDefinition;
}) {
  const activeTone = selectedDefinition ? toneForAddItemKind(selectedDefinition.kind) : "asset-generic";
  const detail = selectedDefinition ? detailForAddItemKind(selectedDefinition.kind) : null;
  const preview = selectedDefinition
    ? createdPathsPreview(selectedDefinition.kind, initialValues(selectedDefinition.kind, scope, prefillRawFilePath))
    : [];

  return (
    <div className="add-item-catalog-layout">
      <div className="add-item-catalog-shell">
        {groups.map((group) => (
          <section className="add-item-catalog-section" key={group.category}>
            <h3>{group.label}</h3>
            <div className="add-item-catalog">
              {group.entries.map((entry) => {
                const tone = toneForAddItemKind(entry.kind);
                const selected = selectedDefinition?.kind === entry.kind;
                return (
                  <button
                    key={entry.kind}
                    className={`add-item-catalog-row add-item-tone ${tone} ${selected ? "selected" : ""}`}
                    type="button"
                    aria-disabled={!entry.enabled}
                    title={entry.enabled ? entry.description : entry.disabledReason}
                    onClick={() => onSelectKind(entry.kind)}
                    onDoubleClick={entry.enabled ? onConfigure : undefined}
                  >
                    <span className={`add-item-catalog-icon add-item-tone ${tone}`}>
                      <entry.icon className={`semantic-icon ${tone}`} size={13} />
                    </span>
                    <span className="add-item-catalog-copy">
                      <strong>{entry.label}</strong>
                      <small>{entry.enabled ? entry.description : entry.disabledReason}</small>
                    </span>
                    {!entry.enabled ? <span className="add-item-status-note">soon</span> : null}
                  </button>
                );
              })}
            </div>
          </section>
        ))}
      </div>

      {selectedDefinition && detail ? (
        <aside className={`add-item-detail-panel add-item-tone ${activeTone}`}>
          <div className="add-item-detail-preview">
            <selectedDefinition.icon className={`semantic-icon ${activeTone}`} size={56} strokeWidth={1.55} />
            <span>Preview pending</span>
          </div>
          <div className="add-item-detail-copy">
            <strong>{selectedDefinition.label}</strong>
            <p>{selectedDefinition.description}</p>
          </div>
          <DetailList title="What it creates" values={detail.creates} />
          <DetailList title="Use this for" values={detail.useFor} />
          {preview.length ? <DetailList title="Default output" values={preview} code /> : null}
          {detail.notes.length ? <DetailList title="Notes" values={detail.notes} /> : null}
          {!selectedDefinition.enabled ? (
            <p className="add-item-detail-status">{selectedDefinition.disabledReason ?? "Coming soon"}</p>
          ) : null}
        </aside>
      ) : null}
    </div>
  );
}

function DetailList({ title, values, code = false }: { title: string; values: string[]; code?: boolean }) {
  return (
    <section className="add-item-detail-list">
      <h4>{title}</h4>
      <ul>
        {values.map((value) => (
          <li key={value}>{code ? <code>{value}</code> : value}</li>
        ))}
      </ul>
    </section>
  );
}
