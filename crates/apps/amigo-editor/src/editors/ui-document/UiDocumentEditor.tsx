import { useMemo, useState } from "react";
import { AlertTriangle, Boxes, FilePlus2, Plus } from "lucide-react";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorUiTemplateKindDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { findUiNode } from "./uiDocumentEditorModel";
import type {
  AddUiDocumentDraft,
  AddUiNodeDraft,
  AddUiTemplateDraft,
  UiNodeCreateKind,
  UiTemplateKind,
} from "./uiDocumentEditorTypes";
import { AddUiDocumentDialog } from "./AddUiDocumentDialog";
import { AddUiNodeDialog } from "./AddUiNodeDialog";
import { AddUiTemplateDialog } from "./AddUiTemplateDialog";
import { UiDocumentChooserPanel } from "./UiDocumentChooserPanel";
import { UiDocumentPreviewPanel } from "./UiDocumentPreviewPanel";
import { UiDocumentStartScreen } from "./UiDocumentStartScreen";
import { firstAllowedUiChildKind, uiNodeCanAddChild } from "./uiNodeCapabilities";
import {
  resolveUiDocumentEditorTarget,
  type UiDocumentEditorTarget,
} from "./uiDocumentTargetResolver";
import "./ui-document-editor.css";

type PendingDialog =
  | { kind: "add-document"; initialTemplate?: UiTemplateKind }
  | { kind: "add-node"; parentPath: string; initialKind?: UiNodeCreateKind }
  | { kind: "add-template"; parentPath: string; initialTemplate?: UiTemplateKind }
  | null;

export function UiDocumentEditor({
  instance,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const [pendingDialog, setPendingDialog] = useState<PendingDialog>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const targetResolution = useMemo(
    () =>
      resolveUiDocumentEditorTarget({
        hierarchy: services.hierarchy ?? null,
        instance,
        services,
      }),
    [
      instance,
      services.details,
      services.hierarchy,
      services.selectedEntity,
      services.selectedScene,
      services.selection,
    ],
  );
  const target: UiDocumentEditorTarget | null =
    targetResolution.kind === "resolved" ? targetResolution.target : null;
  const document = targetResolution.kind === "resolved" ? targetResolution.document : null;
  const activeSceneId =
    target?.sceneId ??
    (targetResolution.kind === "emptyScene" ? targetResolution.sceneId : null) ??
    (targetResolution.kind === "missingTarget" ? targetResolution.sceneId : null) ??
    (targetResolution.kind === "multipleDocuments" ? targetResolution.sceneId : null) ??
    services.selectedScene?.id ??
    null;
  const selectedPath =
    document &&
    services.selection?.kind === "uiNode" &&
    services.selection.nodeRef.entityId === document.entityId &&
    services.selection.nodeRef.componentIndex === document.componentIndex
      ? services.selection.nodeRef.nodePath
      : document?.root.path ?? null;
  const selectedNode = document ? findUiNode(document.root, selectedPath) : null;
  const focusPath =
    document && instance.context?.focusPath
      ? findUiNode(document.root, instance.context.focusPath)?.path ?? null
      : null;

  function selectNode(nodePath: string) {
    if (!document || !services.selectUiNode) return;

    services.selectUiNode({
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath,
    });
  }

  function selectDocument(documentToSelect: { entityId: string; componentIndex: number; root: { path: string } }) {
    services.selectUiNode?.({
      entityId: documentToSelect.entityId,
      componentIndex: documentToSelect.componentIndex,
      nodePath: documentToSelect.root.path,
    });
  }

  function openAddNode(parentPath: string, initialKind?: UiNodeCreateKind) {
    const parentNode = document ? findUiNode(document.root, parentPath) : null;
    if (!uiNodeCanAddChild(parentNode)) {
      setError(parentNode ? `${parentNode.label} cannot contain child nodes.` : "Select a valid parent node.");
      return;
    }

    setError(null);
    setPendingDialog({
      kind: "add-node",
      parentPath,
      initialKind: initialKind ?? firstAllowedUiChildKind(parentNode) ?? undefined,
    });
  }

  function openAddTemplate(parentPath: string, initialTemplate?: UiTemplateKind) {
    setPendingDialog({ kind: "add-template", parentPath, initialTemplate });
  }

  function openCreateDocumentWizard(initialTemplate?: UiTemplateKind) {
    setPendingDialog({ kind: "add-document", initialTemplate });
  }

  function preferredCreateEntityId(): string | null {
    if (targetResolution.kind === "emptyScene") {
      return targetResolution.preferredEntityId ?? null;
    }
    if (targetResolution.kind === "missingTarget") {
      return targetResolution.preferredEntityId ?? targetResolution.entityId ?? null;
    }
    return null;
  }

  async function refreshUiEditorAfterCommand() {
    await services.refreshEditorSnapshot?.();
    await services.refreshSceneHierarchy?.();
  }

  async function runUiCommand(
    command: EditorCommandDto,
    successMessage?: string,
  ): Promise<EditorCommandResultDto | null> {
    if (!services.applyEditorCommand) {
      setError("Editor command service is not available.");
      return null;
    }

    setBusy(true);
    setError(null);

    try {
      const result = await services.applyEditorCommand(command);
      if (!result?.ok) {
        setError(result?.diagnostics.map((diagnostic) => diagnostic.message).join("\n") || result?.message || "UI command failed.");
        return null;
      }

      await refreshUiEditorAfterCommand();

      setPendingDialog(null);
      setNotice(successMessage ?? result.message ?? "UI document updated. Use Save to persist scene changes.");
      return result;
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      return null;
    } finally {
      setBusy(false);
    }
  }

  function selectUiNodeFromCommandResult(result: EditorCommandResultDto | null): boolean {
    const selectedUiNode = result?.snapshot?.selection?.selectedUiNode ?? null;
    if (!selectedUiNode) {
      return false;
    }

    services.selectUiNode?.({
      entityId: selectedUiNode.entityId,
      componentIndex: selectedUiNode.componentIndex,
      nodePath: selectedUiNode.nodePath,
    });

    return true;
  }

  function supportedTemplate(template: UiTemplateKind): EditorUiTemplateKindDto {
    if (
      template === "empty-document" ||
      template === "vertical-menu" ||
      template === "button-row" ||
      template === "health-bar" ||
      template === "ammo-counter" ||
      template === "dialogue-box"
    ) {
      return template;
    }
    return "vertical-menu";
  }

  async function handleCreateDocument(draft: AddUiDocumentDraft) {
    const sceneId = activeSceneId ?? "";
    if (!sceneId) {
      setError("Select a scene before creating a UI document.");
      return;
    }

    const result = await runUiCommand(
      {
        type: "CreateUiDocument",
        sceneId,
        entityId: draft.entityId,
        label: draft.name,
        viewportWidth: draft.viewportWidth,
        viewportHeight: draft.viewportHeight,
        template: supportedTemplate(draft.template),
      },
      `Created UI document "${draft.name}". Use Save to persist scene changes.`,
    );
    if (selectUiNodeFromCommandResult(result)) return;

    const createdDocument = services.hierarchy?.uiDocuments.find(
      (candidate) => candidate.entityId === draft.entityId && candidate.componentIndex === 0,
    );
    if (createdDocument) {
      services.selectUiNode?.({
        entityId: createdDocument.entityId,
        componentIndex: createdDocument.componentIndex,
        nodePath: createdDocument.root.path,
      });
    }
  }

  async function handleCreateNode(draft: AddUiNodeDraft) {
    if (!target || !document) return;

    const result = await runUiCommand(
      {
        type: "AddUiNode",
        sceneId: target.sceneId,
        entityId: document.entityId,
        componentIndex: document.componentIndex,
        parentPath: draft.parentPath,
        node: {
          kind: draft.kind,
          id: draft.id,
          label: draft.label,
          text: draft.text,
        },
        insertIndex: null,
      },
      `Added ${draft.kind} "${draft.id}". Use Save to persist scene changes.`,
    );
    selectUiNodeFromCommandResult(result);
  }

  async function handleCreateTemplate(draft: AddUiTemplateDraft) {
    if (!target || !document) return;

    const result = await runUiCommand(
      {
        type: "AddUiTemplate",
        sceneId: target.sceneId,
        entityId: document.entityId,
        componentIndex: document.componentIndex,
        parentPath: draft.parentPath,
        template: supportedTemplate(draft.template),
        idPrefix: draft.idPrefix,
        insertIndex: null,
      },
      `Added template "${draft.template}". Use Save to persist scene changes.`,
    );
    selectUiNodeFromCommandResult(result);
  }

  function renderAddDocumentDialog() {
    if (pendingDialog?.kind !== "add-document") {
      return null;
    }

    return (
      <AddUiDocumentDialog
        busy={busy}
        initialEntityId={preferredCreateEntityId()}
        initialTemplate={pendingDialog.initialTemplate}
        onClose={() => setPendingDialog(null)}
        onCreate={handleCreateDocument}
      />
    );
  }

  function renderErrorNotice() {
    return error ? (
      <div className="ui-document-notice ui-document-notice-error">
        <AlertTriangle size={14} />
        <span>{error}</span>
        <button type="button" onClick={() => setError(null)}>
          Dismiss
        </button>
      </div>
    ) : null;
  }

  if (targetResolution.kind === "noScene") {
    return (
      <>
        <UiDocumentStartScreen
          mode="noScene"
          message={targetResolution.message}
          onCreateDocument={() => undefined}
        />
        {renderErrorNotice()}
        {renderAddDocumentDialog()}
      </>
    );
  }

  if (targetResolution.kind === "emptyScene") {
    return (
      <>
        <UiDocumentStartScreen
          mode="emptyScene"
          preferredEntityId={targetResolution.preferredEntityId}
          onCreateDocument={(template) => openCreateDocumentWizard(template)}
        />
        {renderErrorNotice()}
        {renderAddDocumentDialog()}
      </>
    );
  }

  if (targetResolution.kind === "missingTarget") {
    return (
      <>
        <UiDocumentStartScreen
          mode="missingTarget"
          message="The selected UI entry is not attached to this scene yet. Create it in the current scene to start editing."
          preferredEntityId={targetResolution.preferredEntityId ?? targetResolution.entityId}
          onCreateDocument={(template) => openCreateDocumentWizard(template)}
        />
        {renderErrorNotice()}
        {renderAddDocumentDialog()}
      </>
    );
  }

  if (targetResolution.kind === "multipleDocuments") {
    return (
      <>
        <UiDocumentChooserPanel
          documents={targetResolution.documents}
          onCreateDocument={() => openCreateDocumentWizard("empty-document")}
          onSelectDocument={selectDocument}
        />

        {renderErrorNotice()}
        {renderAddDocumentDialog()}
      </>
    );
  }

  if (!document) {
    return null;
  }

  const activePath = selectedNode?.path ?? document.root.path;
  return (
    <section className="ui-document-editor">
      <header className="ui-document-editor-header">
        <div>
          <h2>UI Document Editor</h2>
          <span>
            {document.entityName} / UiDocument #{document.componentIndex}
            {focusPath ? ` / Focus: ${focusPath}` : ""}
          </span>
        </div>

        <div className="ui-document-editor-actions">
          <button
            className="button button-ghost"
            type="button"
            onClick={() => openCreateDocumentWizard("empty-document")}
          >
            <FilePlus2 size={15} />
            Add UI Document
          </button>
          <button className="button button-ghost" type="button" onClick={() => openAddNode(activePath)}>
            <Plus size={15} />
            Add Node
          </button>
          <button className="button button-ghost" type="button" onClick={() => openAddTemplate(activePath)}>
            <Boxes size={15} />
            Add Template
          </button>
        </div>
      </header>

      {notice ? (
        <div className="ui-document-notice">
          <AlertTriangle size={14} />
          <span>{notice}</span>
          <button type="button" onClick={() => setNotice(null)}>
            Dismiss
          </button>
        </div>
      ) : null}

      {error ? (
        <div className="ui-document-notice ui-document-notice-error">
          <AlertTriangle size={14} />
          <span>{error}</span>
          <button type="button" onClick={() => setError(null)}>
            Dismiss
          </button>
        </div>
      ) : null}

      <div className="ui-document-editor-body">
        <main className="ui-document-center">
          <UiDocumentPreviewPanel
            document={document}
            focusPath={focusPath}
            selectedNode={selectedNode}
            onSelectNode={selectNode}
          />
          <footer className="ui-document-breadcrumb">
            <span>UiDocument</span>
            {activePath.split(".").map((part, index, parts) => (
              <button key={`${part}:${index}`} type="button" onClick={() => selectNode(parts.slice(0, index + 1).join("."))}>
                {part}
              </button>
            ))}
          </footer>
        </main>

      </div>

      {pendingDialog?.kind === "add-document" ? (
        <AddUiDocumentDialog
          busy={busy}
          initialEntityId={preferredCreateEntityId()}
          initialTemplate={pendingDialog.initialTemplate}
          onClose={() => setPendingDialog(null)}
          onCreate={handleCreateDocument}
        />
      ) : null}

      {pendingDialog?.kind === "add-node" ? (
        <AddUiNodeDialog
          busy={busy}
          initialKind={pendingDialog.initialKind}
          parentNode={document ? findUiNode(document.root, pendingDialog.parentPath) : null}
          parentPath={pendingDialog.parentPath}
          onClose={() => setPendingDialog(null)}
          onCreate={handleCreateNode}
        />
      ) : null}

      {pendingDialog?.kind === "add-template" ? (
        <AddUiTemplateDialog
          busy={busy}
          initialTemplate={pendingDialog.initialTemplate}
          parentPath={pendingDialog.parentPath}
          onClose={() => setPendingDialog(null)}
          onCreate={handleCreateTemplate}
        />
      ) : null}

    </section>
  );
}
