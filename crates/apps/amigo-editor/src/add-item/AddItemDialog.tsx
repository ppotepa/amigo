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

function toneForAddItemKind(kind: AddItemKind): string {
  switch (kind) {
    case "scene":
      return "asset-scene";
    case "ui-theme":
      return "domain-theme";
    case "script":
      return "asset-script";
    case "folder":
      return "domain-project";
    case "raw-source":
      return "asset-raw-image";
    case "font":
      return "asset-font";
    case "image":
      return "asset-image";
    case "spritesheet":
      return "asset-sprite";
    case "tileset":
      return "asset-tileset";
    case "tilemap":
      return "asset-tilemap";
    case "audio":
      return "asset-audio";
    case "material":
    case "mesh":
      return "domain-rendering_2d";
    case "prefab":
    default:
      return "asset-generic";
  }
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

const ADD_ITEM_CATEGORY_LABELS: Record<string, string> = {
  project: "Project",
  assets: "Assets",
  ui: "UI",
  scripts: "Scripts",
  advanced: "Advanced",
};

function detailForAddItemKind(kind: AddItemKind): {
  creates: string[];
  useFor: string[];
  notes: string[];
} {
  switch (kind) {
    case "scene":
      return {
        creates: ["scene.yml document", "Optional scene.rhai script", "mod.toml scene entry"],
        useFor: ["Gameplay levels", "Runtime menus", "Previewable scene states"],
        notes: ["Best default choice when adding a new playable or previewable area."],
      };
    case "ui-theme":
      return {
        creates: ["UI theme YAML descriptor"],
        useFor: ["Runtime UI palettes", "Menu visual themes", "Reusable UI color sets"],
        notes: ["Theme editing is descriptor-first until the visual UI editor lands."],
      };
    case "script":
      return {
        creates: ["Rhai script file"],
        useFor: ["Scene logic", "Runtime event hooks", "Prototype behavior"],
        notes: ["Scene-owned scripts should usually live beside their scene."],
      };
    case "folder":
      return {
        creates: ["Project folder"],
        useFor: ["Organizing source assets", "Grouping descriptors", "Preparing future asset domains"],
        notes: ["Folders are created only inside the active mod project."],
      };
    case "raw-source":
      return {
        creates: ["Copied source file in raw/"],
        useFor: ["Imported art/audio/source material", "Files that will later become typed assets"],
        notes: ["Raw files are not runtime descriptors until a typed asset references them."],
      };
    case "font":
      return {
        creates: ["fonts/<id>/font.yml"],
        useFor: ["Text2D", "Runtime UI text", "Menu typography"],
        notes: ["The first version creates a placeholder descriptor."],
      };
    case "image":
      return {
        creates: ["Image, sprite, or tileset descriptor from a raw image"],
        useFor: ["2D backgrounds", "Spritesheets", "Tileset sources"],
        notes: ["Requires selecting an existing raw image source."],
      };
    case "spritesheet":
      return {
        creates: ["Spritesheet descriptor"],
        useFor: ["Frame grids", "Animated 2D sprites", "Sprite libraries"],
        notes: ["Coming soon."],
      };
    case "tileset":
      return {
        creates: ["Tileset descriptor"],
        useFor: ["Tile palettes", "Tilemap painting", "Grid based maps"],
        notes: ["Coming soon."],
      };
    case "tilemap":
      return {
        creates: ["Tilemap data document"],
        useFor: ["Level collision/layout maps", "Tile based scenes", "2D tactical maps"],
        notes: ["Coming soon."],
      };
    case "audio":
      return {
        creates: ["Audio descriptor"],
        useFor: ["Music", "SFX", "Runtime audio references"],
        notes: ["Coming soon."],
      };
    case "prefab":
      return {
        creates: ["Prefab document"],
        useFor: ["Reusable entity trees", "UI widgets", "Spawnable game objects"],
        notes: ["Coming soon."],
      };
    case "material":
      return {
        creates: ["Material descriptor"],
        useFor: ["3D rendering", "Shader/material references", "Reusable visual surfaces"],
        notes: ["Coming soon."],
      };
    case "mesh":
      return {
        creates: ["Mesh descriptor"],
        useFor: ["3D models", "Imported geometry", "Reusable mesh assets"],
        notes: ["Coming soon."],
      };
    default:
      return {
        creates: ["Project item"],
        useFor: ["Project content"],
        notes: [],
      };
  }
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
  const [selectedCatalogKind, setSelectedCatalogKind] = useState<AddItemKind>(() => {
    const preferred = request.itemKind ?? defaultKindForScope(request.scope);
    if (preferred) return preferred;
    return scopedCatalog.find((entry) => entry.enabled)?.kind ?? scopedCatalog[0]?.kind ?? "scene";
  });
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
  const selectedCatalogDefinition = scopedCatalog.find((entry) => entry.kind === selectedCatalogKind) ?? scopedCatalog[0];
  const showCatalog = request.mode === "catalog" && !kind;
  const displayDefinition = showCatalog ? selectedCatalogDefinition : selectedDefinition;
  const selectedTone = displayDefinition ? toneForAddItemKind(displayDefinition.kind) : undefined;
  const activeTone = selectedTone ?? "asset-generic";
  const selectedKind = kind ?? "scene";
  const outputPreview = createdPathsPreview(selectedKind, values);
  const catalogDetail = selectedCatalogDefinition ? detailForAddItemKind(selectedCatalogDefinition.kind) : null;
  const catalogPreview = selectedCatalogDefinition
    ? createdPathsPreview(selectedCatalogDefinition.kind, initialValues(selectedCatalogDefinition.kind, details, request.scope, request.prefillRawFilePath))
    : [];
  const catalogGroups = ["project", "ui", "scripts", "assets", "advanced"]
    .map((category) => ({
      category,
      label: ADD_ITEM_CATEGORY_LABELS[category] ?? category,
      entries: scopedCatalog.filter((entry) => entry.category === category),
    }))
    .filter((group) => group.entries.length > 0);
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

  function selectKind(nextKind: AddItemKind) {
    const definition = scopedCatalog.find((entry) => entry.kind === nextKind);
    if (!definition || !definition.enabled) return;
    const initial = initialValues(nextKind, details, request.scope, request.prefillRawFilePath);
    if (request.prefillDescriptorKind) {
      initial.descriptorKind = request.prefillDescriptorKind;
    }
    initial.itemId = resolveInitialItemId(nextKind, initial, isItemIdTaken);
    setKind(nextKind);
    setValues(initial);
    setError(null);
  }

  function configureSelectedCatalogKind() {
    if (!selectedCatalogDefinition?.enabled) return;
    selectKind(selectedCatalogDefinition.kind);
  }

  return (
    <AppDialog
      title="Add Item"
      titleTag={showCatalog ? undefined : (selectedDefinition?.label ?? "Item")}
      subtitle={`Create a new item in ${details.name}.`}
      icon={displayDefinition ? <displayDefinition.icon className={`semantic-icon ${selectedTone}`} size={18} /> : undefined}
      iconClassName={selectedTone ? `add-item-tone ${selectedTone}` : undefined}
      toneClassName={selectedTone ? `add-item-tone ${selectedTone}` : "dialog-tone-project"}
      headerVariant="windows"
      onClose={onCancel}
      closeDisabled={busy}
      backdropClassName="add-item-backdrop new-project-backdrop"
      dialogClassName={`new-project-dialog add-item-dialog ${showCatalog ? "add-item-dialog-catalog" : ""}`}
      bodyClassName="new-project-body add-item-body"
      footer={(
        <>
          <button className="button button-ghost" type="button" disabled={busy} onClick={onCancel}>Cancel</button>
          {showCatalog ? (
            <button
              className="button button-primary"
              type="button"
              disabled={!selectedCatalogDefinition?.enabled}
              onClick={configureSelectedCatalogKind}
            >
              Configure
            </button>
          ) : (
            <button className="button button-primary" type="button" disabled={!canCreate} onClick={() => void handleSubmit()}>
              {busy ? "Creating..." : "Create"}
            </button>
          )}
        </>
      )}
    >
          {showCatalog ? (
            <div className="add-item-catalog-layout">
              <div className="add-item-catalog-shell">
                {catalogGroups.map((group) => (
                  <section className="add-item-catalog-section" key={group.category}>
                    <h3>{group.label}</h3>
                    <div className="add-item-catalog">
                      {group.entries.map((entry) => {
                        const tone = toneForAddItemKind(entry.kind);
                        const selected = selectedCatalogDefinition?.kind === entry.kind;
                        return (
                          <button
                            key={entry.kind}
                            className={`add-item-catalog-row add-item-tone ${tone} ${selected ? "selected" : ""}`}
                            type="button"
                            aria-disabled={!entry.enabled}
                            title={entry.enabled ? entry.description : entry.disabledReason}
                            onClick={() => setSelectedCatalogKind(entry.kind)}
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
              {selectedCatalogDefinition && catalogDetail ? (
                <aside className={`add-item-detail-panel add-item-tone ${activeTone}`}>
                  <div className="add-item-detail-preview">
                    <selectedCatalogDefinition.icon className={`semantic-icon ${activeTone}`} size={56} strokeWidth={1.55} />
                    <span>Preview pending</span>
                  </div>
                  <div className="add-item-detail-copy">
                    <strong>{selectedCatalogDefinition.label}</strong>
                    <p>{selectedCatalogDefinition.description}</p>
                  </div>
                  <DetailList title="What it creates" values={catalogDetail.creates} />
                  <DetailList title="Use this for" values={catalogDetail.useFor} />
                  {catalogPreview.length ? <DetailList title="Default output" values={catalogPreview} code /> : null}
                  {catalogDetail.notes.length ? <DetailList title="Notes" values={catalogDetail.notes} /> : null}
                  {!selectedCatalogDefinition.enabled ? (
                    <p className="add-item-detail-status">{selectedCatalogDefinition.disabledReason ?? "Coming soon"}</p>
                  ) : null}
                </aside>
              ) : null}
            </div>
          ) : (
            <div className="add-item-compact">
              {selectedDefinition ? (
                <div className={`add-item-selected-type add-item-tone ${activeTone}`}>
                  <span className={`add-item-catalog-icon add-item-tone ${activeTone}`}>
                    <selectedDefinition.icon className={`semantic-icon ${activeTone}`} size={16} />
                  </span>
                  <div>
                    <strong>{selectedDefinition.label}</strong>
                    <small>{selectedDefinition.description}</small>
                  </div>
                  {request.mode === "catalog" ? (
                    <button className="button button-ghost add-item-change-type" type="button" disabled={busy} onClick={() => setKind(undefined)}>
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
                      className={idValidation.invalid ? "is-invalid" : ""}
                      aria-invalid={idValidation.invalid}
                      value={values.itemId}
                      onChange={(event) => {
                        setValues((current) => ({ ...current, itemId: event.target.value }));
                        setError(null);
                      }}
                    />
                    {idValidation.invalid ? <small>{idValidation.error}</small> : null}
                  </div>
                </label>

                <label className="add-item-row">
                  <span className="add-item-label">Target</span>
                  <div className="add-item-control">
                    <input value={values.targetFolder} onChange={(event) => setValues((current) => ({ ...current, targetFolder: event.target.value }))} />
                  </div>
                </label>

                <label className="add-item-row">
                  <span className="add-item-label">Label</span>
                  <div className="add-item-control">
                    <input value={values.label} onChange={(event) => setValues((current) => ({ ...current, label: event.target.value }))} />
                  </div>
                </label>

                {kind === "raw-source" || kind === "image" ? (
                  <label className="add-item-row">
                    <span className="add-item-label">Source</span>
                    <div className="add-item-control add-item-control-inline">
                      <input value={values.sourceFilePath} onChange={(event) => setValues((current) => ({ ...current, sourceFilePath: event.target.value }))} />
                      {onPickSourceFile ? (
                        <button
                          type="button"
                          className="button button-ghost add-item-browse"
                          onClick={() => {
                            void onPickSourceFile().then((picked) => {
                              if (!picked) return;
                              setValues((current) => ({ ...current, sourceFilePath: picked }));
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
                      <select value={values.descriptorKind} onChange={(event) => setValues((current) => ({ ...current, descriptorKind: event.target.value as "image" | "sprite" | "tileset" }))}>
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
                    <input type="checkbox" checked={values.createScript} onChange={(event) => setValues((current) => ({ ...current, createScript: event.target.checked }))} />
                    <span>Create scene.rhai</span>
                  </label>
                  <label className="add-item-toggle-row">
                    <input type="checkbox" checked={values.launcherVisible} onChange={(event) => setValues((current) => ({ ...current, launcherVisible: event.target.checked }))} />
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
          )}
          {error ? <p className="muted workspace-note">{error}</p> : null}
    </AppDialog>
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
