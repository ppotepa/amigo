import { useMemo, useState } from "react";
import type { AddItemScope, AddItemDialogRequest, AddItemKind, AddItemFormValue } from "./addItemTypes";
import { ADD_ITEM_DEFINITIONS, catalogForScope, defaultKindForScope } from "./addItemCatalog";
import type { CreateAssetImportOptionsDto, EditorModDetailsDto } from "../api/dto";
import { AppDialog } from "../ui/dialog/AppDialog";
import { nextAvailableSlugId, normalizeSlugId, validateSlugId } from "../ui/validation/slugId";

function initialTargetFolder(kind: AddItemKind, scope?: AddItemScope): string {
  if (scope?.kind === "project-folder") return scope.path;
  const definition = ADD_ITEM_DEFINITIONS.find((entry) => entry.kind === kind);
  return definition?.defaultTargetPath ?? "";
}

function initialValues(kind: AddItemKind, details: EditorModDetailsDto, scope?: AddItemScope, prefillRawFilePath?: string): AddItemFormValue {
  return {
    itemId: defaultItemIdForKind(kind),
    label: kind === "scene" ? "New Scene" : "New Item",
    targetFolder: initialTargetFolder(kind, scope),
    createScript: kind === "scene",
    launcherVisible: false,
    sourceFilePath: prefillRawFilePath ?? "",
    descriptorKind: "image",
    importOptions: {
      tileWidth: 32,
      tileHeight: 32,
      columns: 1,
      rows: 1,
      tileCount: 1,
    },
  };
}

function defaultItemIdForKind(kind: AddItemKind): string {
  if (kind === "scene") return "new-scene";
  if (kind === "font") return "new-font";
  if (kind === "script") return "new-script";
  return "new-item";
}

function resolveInitialItemId(
  kind: AddItemKind,
  values: AddItemFormValue,
  isItemIdTaken?: (payload: {
    kind: AddItemKind;
    itemId: string;
    targetFolder: string;
    descriptorKind: "image" | "sprite" | "tileset";
  }) => boolean,
): string {
  if (!isItemIdTaken) return defaultItemIdForKind(kind);
  return nextAvailableSlugId(defaultItemIdForKind(kind), (candidate) => (
    isItemIdTaken({
      kind,
      itemId: candidate,
      targetFolder: values.targetFolder.trim(),
      descriptorKind: values.descriptorKind,
    })
  ));
}

function joinPath(base: string, tail: string): string {
  if (!base.trim()) return tail;
  return `${base.replace(/\/+$/, "")}/${tail.replace(/^\/+/, "")}`;
}

function sourceFileName(sourcePath: string): string {
  const normalized = sourcePath.split("\\").join("/");
  const last = normalized.split("/").pop()?.trim();
  return last || "source.file";
}

function createdPathsPreview(kind: AddItemKind, values: AddItemFormValue): string[] {
  const id = normalizeSlugId(values.itemId);
  if (!id) return [];

  if (kind === "scene") {
    const root = joinPath(values.targetFolder || "scenes", id);
    const files = [`${root}/scene.yml`];
    if (values.createScript) files.push(`${root}/scene.rhai`);
    return files;
  }

  if (kind === "script") {
    return [joinPath(values.targetFolder || "scripts", `${id}.rhai`)];
  }

  if (kind === "font") {
    return [joinPath(values.targetFolder || "fonts", `${id}/font.yml`)];
  }

  if (kind === "ui-theme") {
    return [joinPath(values.targetFolder || "ui/themes", `${id}.yml`)];
  }

  if (kind === "raw-source") {
    return [joinPath(values.targetFolder || "raw", sourceFileName(values.sourceFilePath))];
  }

  if (kind === "folder") {
    return [`${joinPath(values.targetFolder, id)}/`];
  }

  if (kind === "image") {
    return [`descriptor:${values.descriptorKind}:${id}`];
  }

  return [];
}

export function AddItemDialog({
  details,
  request,
  onCancel,
  onCreateProjectItem,
  onCreateDescriptor,
  onPickSourceFile,
  isItemIdTaken,
}: {
  details: EditorModDetailsDto;
  request: AddItemDialogRequest;
  onCancel: () => void;
  onCreateProjectItem: (payload: {
    kind: AddItemKind;
    itemId: string;
    label: string;
    targetFolder: string;
    createScript: boolean;
    launcherVisible: boolean;
    sourceFilePath?: string;
  }) => Promise<void>;
  onCreateDescriptor: (payload: {
    rawFilePath: string;
    kind: "image" | "sprite" | "tileset";
    assetId: string;
    importOptions?: CreateAssetImportOptionsDto | null;
  }) => Promise<void>;
  onPickSourceFile?: () => Promise<string | null>;
  isItemIdTaken?: (payload: {
    kind: AddItemKind;
    itemId: string;
    targetFolder: string;
    descriptorKind: "image" | "sprite" | "tileset";
  }) => boolean;
}) {
  const scopedCatalog = useMemo(() => catalogForScope(request.scope), [request.scope]);
  const [kind, setKind] = useState<AddItemKind | undefined>(() => request.itemKind ?? defaultKindForScope(request.scope));
  const [values, setValues] = useState<AddItemFormValue>(() => {
    const selectedKind = request.itemKind ?? defaultKindForScope(request.scope) ?? "scene";
    const initial = initialValues(
      selectedKind,
      details,
      request.scope,
      request.prefillRawFilePath,
    );
    if (request.prefillDescriptorKind) {
      initial.descriptorKind = request.prefillDescriptorKind;
    }
    initial.itemId = resolveInitialItemId(selectedKind, initial, isItemIdTaken);
    return initial;
  });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectedDefinition = ADD_ITEM_DEFINITIONS.find((entry) => entry.kind === kind);
  const showCatalog = request.mode === "catalog" && !kind;
  const selectedKind = kind ?? "scene";
  const outputPreview = createdPathsPreview(selectedKind, values);
  const idValidation = validateSlugId(
    values.itemId,
    kind ? Boolean(isItemIdTaken?.({
      kind,
      itemId: normalizeSlugId(values.itemId),
      targetFolder: values.targetFolder.trim(),
      descriptorKind: values.descriptorKind,
    })) : false,
  );
  const canCreate = !busy && Boolean(kind) && !idValidation.invalid;

  async function handleSubmit() {
    if (!kind) return;
    const normalizedId = idValidation.normalized;
    if (idValidation.invalid) {
      setError(idValidation.error ?? "Invalid item id.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      if (kind === "image") {
        if (!values.sourceFilePath.trim()) {
          throw new Error("Raw file path is required for image descriptor.");
        }
        const options = values.descriptorKind === "image" ? null : values.importOptions;
        await onCreateDescriptor({
          rawFilePath: values.sourceFilePath.trim(),
          kind: values.descriptorKind,
          assetId: normalizedId,
          importOptions: options,
        });
      } else {
        await onCreateProjectItem({
          kind,
          itemId: normalizedId,
          label: values.label.trim(),
          targetFolder: values.targetFolder.trim(),
          createScript: values.createScript,
          launcherVisible: values.launcherVisible,
          sourceFilePath: values.sourceFilePath.trim() || undefined,
        });
      }
      onCancel();
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : String(submitError));
    } finally {
      setBusy(false);
    }
  }

  return (
    <AppDialog
      title={showCatalog ? "Add Item" : `Add ${selectedDefinition?.label ?? "Item"}`}
      subtitle={`Create a new item in ${details.name}.`}
      icon={selectedDefinition ? <selectedDefinition.icon size={18} /> : undefined}
      onClose={onCancel}
      closeDisabled={busy}
      backdropClassName="add-item-backdrop new-project-backdrop"
      footer={(
        <>
          {kind && request.mode === "catalog" ? (
            <button className="button button-ghost" type="button" disabled={busy} onClick={() => setKind(undefined)}>
              Back
            </button>
          ) : null}
          <button className="button button-ghost" type="button" disabled={busy} onClick={onCancel}>Cancel</button>
          <button className="button button-primary" type="button" disabled={!canCreate} onClick={() => void handleSubmit()}>
            {busy ? "Creating..." : "Create"}
          </button>
        </>
      )}
    >
          {showCatalog ? (
            <div className="project-type-grid">
              {scopedCatalog.map((entry) => {
                const Icon = entry.icon;
                return (
                  <button
                    key={entry.kind}
                    className={`project-type-card ${!entry.enabled ? "disabled" : ""}`}
                    disabled={!entry.enabled}
                    type="button"
                    onClick={() => {
                      setKind(entry.kind);
                      const initial = initialValues(entry.kind, details, request.scope, request.prefillRawFilePath);
                      if (request.prefillDescriptorKind) {
                        initial.descriptorKind = request.prefillDescriptorKind;
                      }
                      initial.itemId = resolveInitialItemId(entry.kind, initial, isItemIdTaken);
                      setValues(initial);
                      setError(null);
                    }}
                  >
                    <span className="project-type-icon"><Icon size={18} /></span>
                    <strong>{entry.label}</strong>
                    <small>{entry.enabled ? entry.description : entry.disabledReason}</small>
                  </button>
                );
              })}
            </div>
          ) : (
            <div className="add-item-layout">
              <div className="new-project-form add-item-form">
                <label className="new-project-field">
                  <span>Item id</span>
                  <input
                    className={idValidation.invalid ? "is-invalid" : ""}
                    aria-invalid={idValidation.invalid}
                    value={values.itemId}
                    onChange={(event) => {
                      setValues((current) => ({ ...current, itemId: event.target.value }));
                      setError(null);
                    }}
                  />
                  {idValidation.invalid ? <small>{idValidation.error}</small> : null}
                </label>
                <label className="new-project-field">
                  <span>Label</span>
                  <input value={values.label} onChange={(event) => setValues((current) => ({ ...current, label: event.target.value }))} />
                </label>
                <label className="new-project-field">
                  <span>Target folder</span>
                  <input value={values.targetFolder} onChange={(event) => setValues((current) => ({ ...current, targetFolder: event.target.value }))} />
                </label>

                {kind === "scene" ? (
                  <div className="add-item-toggle-group">
                    <label className="add-item-toggle-row">
                      <input type="checkbox" checked={values.createScript} onChange={(event) => setValues((current) => ({ ...current, createScript: event.target.checked }))} />
                      <span>Create scene.rhai</span>
                    </label>
                    <label className="add-item-toggle-row">
                      <input type="checkbox" checked={values.launcherVisible} onChange={(event) => setValues((current) => ({ ...current, launcherVisible: event.target.checked }))} />
                      <span>Visible in launcher</span>
                    </label>
                  </div>
                ) : null}

                {kind === "raw-source" || kind === "image" ? (
                  <>
                    <label className="new-project-field">
                      <span>Source path</span>
                      <input value={values.sourceFilePath} onChange={(event) => setValues((current) => ({ ...current, sourceFilePath: event.target.value }))} />
                    </label>
                    {onPickSourceFile ? (
                      <label className="new-project-field">
                        <span />
                        <button
                          type="button"
                          className="button button-ghost"
                          onClick={() => {
                            void onPickSourceFile().then((picked) => {
                              if (!picked) return;
                              setValues((current) => ({ ...current, sourceFilePath: picked }));
                            });
                          }}
                        >
                          Browse...
                        </button>
                      </label>
                    ) : null}
                  </>
                ) : null}

                {kind === "image" ? (
                  <label className="new-project-field">
                    <span>Descriptor kind</span>
                    <select value={values.descriptorKind} onChange={(event) => setValues((current) => ({ ...current, descriptorKind: event.target.value as "image" | "sprite" | "tileset" }))}>
                      <option value="image">image</option>
                      <option value="sprite">sprite</option>
                      <option value="tileset">tileset</option>
                    </select>
                  </label>
                ) : null}
              </div>
              <aside className="add-item-summary">
                <h3>{selectedDefinition?.label ?? "Item"}</h3>
                <p>{selectedDefinition?.description ?? "Create a new item."}</p>
                <div className="add-item-summary-block">
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
                </div>
              </aside>
            </div>
          )}
          {error ? <p className="muted workspace-note">{error}</p> : null}
    </AppDialog>
  );
}
