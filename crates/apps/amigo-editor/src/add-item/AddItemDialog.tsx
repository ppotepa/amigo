import { useMemo, useState } from "react";
import type { AddItemDialogRequest, AddItemKind, AddItemFormValue } from "./addItemTypes";
import { ADD_ITEM_DEFINITIONS, catalogForScope, defaultKindForScope } from "./addItemCatalog";
import { AddItemCatalogView } from "./AddItemCatalogView";
import { AddItemFormHost } from "./AddItemFormHost";
import type { CreateAssetImportOptionsDto, EditorModDetailsDto } from "../api/dto";
import { AppDialog } from "../ui/dialog/AppDialog";
import { OperationScopeHint } from "../ui/feedback/OperationScopeHint";
import { normalizeSlugId, validateSlugId } from "../ui/validation/slugId";
import {
  createdPathsPreview,
  initialValues,
  resolveInitialItemId,
} from "./addItemDraft";
import {
  ADD_ITEM_CATEGORY_LABELS,
  toneForAddItemKind,
} from "./addItemPresentation";

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
    const initial = initialValues(nextKind, request.scope, request.prefillRawFilePath);
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
          {!showCatalog ? <OperationScopeHint kind="project-item" /> : null}
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
            <AddItemCatalogView
              groups={catalogGroups}
              onConfigure={configureSelectedCatalogKind}
              onSelectKind={setSelectedCatalogKind}
              prefillRawFilePath={request.prefillRawFilePath}
              scope={request.scope}
              selectedDefinition={selectedCatalogDefinition}
            />
          ) : (
            <AddItemFormHost
              busy={busy}
              idInvalid={idValidation.invalid}
              idValidationError={idValidation.error ?? undefined}
              kind={selectedKind}
              onBackToCatalog={request.mode === "catalog" ? () => setKind(undefined) : undefined}
              onPickSourceFile={onPickSourceFile}
              onValuesChange={(next) => {
                setValues(next);
                setError(null);
              }}
              outputPreview={outputPreview}
              selectedDefinition={selectedDefinition}
              tone={activeTone}
              values={values}
            />
          )}
          {error ? <p className="muted workspace-note">{error}</p> : null}
    </AppDialog>
  );
}
