import { useMemo, useState } from "react";
import { AlertTriangle, Boxes, FilePlus2, Plus } from "lucide-react";
import type {
  EditorCommandDto,
  EditorUiNodeMoveDirectionDto,
  EditorUiTemplateKindDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { canHaveChildren, findUiNode, getSiblingInfo } from "./uiDocumentEditorModel";
import type {
  AddUiDocumentDraft,
  AddUiNodeDraft,
  AddUiTemplateDraft,
  UiDocumentEditorTab,
  UiNodeCreateKind,
  UiTemplateKind,
} from "./uiDocumentEditorTypes";
import { AddUiDocumentDialog } from "./AddUiDocumentDialog";
import { AddUiNodeDialog } from "./AddUiNodeDialog";
import { AddUiTemplateDialog } from "./AddUiTemplateDialog";
import { ConfirmRemoveUiNodeDialog } from "./ConfirmRemoveUiNodeDialog";
import { UiDocumentChooserPanel } from "./UiDocumentChooserPanel";
import { UiDocumentInspectorPanel } from "./UiDocumentInspectorPanel";
import { UiDocumentPreviewPanel } from "./UiDocumentPreviewPanel";
import { UiDocumentStartScreen } from "./UiDocumentStartScreen";
import { UiDocumentTreePanel } from "./UiDocumentTreePanel";
import {
  resolveUiDocumentEditorTarget,
  type UiDocumentEditorTarget,
} from "./uiDocumentTargetResolver";
import { UiNodePalettePanel } from "./UiNodePalettePanel";
import { UiTemplatePanel } from "./UiTemplatePanel";
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
  const [activeLeftTab, setActiveLeftTab] = useState<UiDocumentEditorTab>("tree");
  const [pendingDialog, setPendingDialog] = useState<PendingDialog>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmRemovePath, setConfirmRemovePath] = useState<string | null>(null);

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
  const selectedPath =
    document &&
    services.selection?.kind === "uiNode" &&
    services.selection.nodeRef.entityId === document.entityId &&
    services.selection.nodeRef.componentIndex === document.componentIndex
      ? services.selection.nodeRef.nodePath
      : document?.root.path ?? null;
  const selectedNode = document ? findUiNode(document.root, selectedPath) : null;

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
    setPendingDialog({ kind: "add-node", parentPath, initialKind });
  }

  function openAddTemplate(parentPath: string, initialTemplate?: UiTemplateKind) {
    setPendingDialog({ kind: "add-template", parentPath, initialTemplate });
  }

  async function refreshUiEditorAfterCommand() {
    await services.refreshEditorSnapshot?.();
    await services.refreshSceneHierarchy?.();
  }

  async function runUiCommand(command: EditorCommandDto, successMessage?: string): Promise<boolean> {
    if (!services.applyEditorCommand) {
      setError("Editor command service is not available.");
      return false;
    }

    setBusy(true);
    setError(null);

    try {
      const result = await services.applyEditorCommand(command);
      if (!result?.ok) {
        setError(result?.diagnostics.map((diagnostic) => diagnostic.message).join("\n") || result?.message || "UI command failed.");
        return false;
      }

      await refreshUiEditorAfterCommand();

      setPendingDialog(null);
      setConfirmRemovePath(null);
      setNotice(successMessage ?? result.message ?? "UI document updated. Use Save to persist scene changes.");
      return true;
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      return false;
    } finally {
      setBusy(false);
    }
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
    const sceneId = target?.sceneId ?? services.selectedScene?.id ?? "";
    if (!sceneId) {
      setError("Select a scene before creating a UI document.");
      return;
    }

    const ok = await runUiCommand(
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
    if (!ok) return;

    services.selectUiNode?.({
      entityId: draft.entityId,
      componentIndex: 0,
      nodePath: "root",
    });
  }

  async function handleCreateNode(draft: AddUiNodeDraft) {
    if (!target || !document) return;

    const ok = await runUiCommand(
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
    if (!ok) return;

    services.selectUiNode?.({
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath: `${draft.parentPath}.${draft.id}`,
    });
  }

  async function handleCreateTemplate(draft: AddUiTemplateDraft) {
    if (!target || !document) return;

    const ok = await runUiCommand(
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
    if (!ok) return;

    services.selectUiNode?.({
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath: draft.parentPath,
    });
  }

  async function runNodeCommand(direction?: EditorUiNodeMoveDirectionDto) {
    if (!target || !document || !selectedNode) return;
    if (direction) {
      await runUiCommand(
        {
          type: "MoveUiNode",
          sceneId: target.sceneId,
          entityId: document.entityId,
          componentIndex: document.componentIndex,
          nodePath: selectedNode.path,
          direction,
        },
        `Moved "${selectedNode.path}" ${direction}. Use Save to persist scene changes.`,
      );
      return;
    }

    const ok = await runUiCommand(
      {
        type: "DuplicateUiNode",
        sceneId: target.sceneId,
        entityId: document.entityId,
        componentIndex: document.componentIndex,
        nodePath: selectedNode.path,
        newId: null,
        copyActions: false,
      },
      `Duplicated "${selectedNode.path}". Use Save to persist scene changes.`,
    );
  }

  async function confirmRemoveNode() {
    if (!target || !document || !confirmRemovePath) return;
    const removedPath = confirmRemovePath;
    const ok = await runUiCommand(
      {
        type: "RemoveUiNode",
        sceneId: target.sceneId,
        entityId: document.entityId,
        componentIndex: document.componentIndex,
        nodePath: removedPath,
      },
      `Removed "${removedPath}". Use Save to persist scene changes.`,
    );
    if (!ok) return;

    services.selectUiNode?.({
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath: parentPathOf(removedPath),
    });
  }

  if (targetResolution.kind === "ambiguous") {
    return (
      <>
        <UiDocumentChooserPanel
          documents={targetResolution.documents}
          onCreateDocument={() => setPendingDialog({ kind: "add-document", initialTemplate: "empty-document" })}
          onSelectDocument={selectDocument}
        />

        {pendingDialog?.kind === "add-document" ? (
          <AddUiDocumentDialog
            busy={busy}
            initialTemplate={pendingDialog.initialTemplate}
            onClose={() => setPendingDialog(null)}
            onCreate={handleCreateDocument}
          />
        ) : null}
      </>
    );
  }

  if (!document) {
    return (
      <>
        <UiDocumentStartScreen
          onCreateBlank={() => setPendingDialog({ kind: "add-document", initialTemplate: "empty-document" })}
          onCreateFromTemplate={(template) => setPendingDialog({ kind: "add-document", initialTemplate: template })}
        />

        {error ? (
          <div className="ui-document-notice ui-document-notice-error">
            <AlertTriangle size={14} />
            <span>{error}</span>
            <button type="button" onClick={() => setError(null)}>
              Dismiss
            </button>
          </div>
        ) : null}

        {pendingDialog?.kind === "add-document" ? (
          <AddUiDocumentDialog
            busy={busy}
            initialTemplate={pendingDialog.initialTemplate}
            onClose={() => setPendingDialog(null)}
            onCreate={handleCreateDocument}
          />
        ) : null}
      </>
    );
  }

  const activePath = selectedNode?.path ?? document.root.path;
  const siblingInfo = getSiblingInfo(document.root, activePath);
  const canAddChild = Boolean(selectedNode && canHaveChildren(selectedNode.kind));
  const canDuplicate = Boolean(selectedNode && selectedNode.path !== document.root.path);
  const canRemove = canDuplicate;
  const canMoveUp = Boolean(siblingInfo && siblingInfo.index > 0);
  const canMoveDown = Boolean(siblingInfo && siblingInfo.index + 1 < siblingInfo.count);
  const confirmRemoveNodeValue = confirmRemovePath ? findUiNode(document.root, confirmRemovePath) : null;

  return (
    <section className="ui-document-editor">
      <header className="ui-document-editor-header">
        <div>
          <h2>UI Document Editor</h2>
          <span>
            {document.entityName} / UiDocument #{document.componentIndex}
          </span>
        </div>

        <div className="ui-document-editor-actions">
          <button
            className="button button-ghost"
            type="button"
            onClick={() => setPendingDialog({ kind: "add-document", initialTemplate: "empty-document" })}
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
        <aside className="ui-document-left">
          <nav className="ui-document-left-tabs">
            <button className={activeLeftTab === "tree" ? "active" : ""} type="button" onClick={() => setActiveLeftTab("tree")}>
              Tree
            </button>
            <button className={activeLeftTab === "palette" ? "active" : ""} type="button" onClick={() => setActiveLeftTab("palette")}>
              Palette
            </button>
            <button className={activeLeftTab === "templates" ? "active" : ""} type="button" onClick={() => setActiveLeftTab("templates")}>
              Templates
            </button>
          </nav>

          {activeLeftTab === "tree" ? (
            <UiDocumentTreePanel
              document={document}
              selectedPath={activePath}
              onAddChild={(parentPath) => openAddNode(parentPath)}
              onSelectNode={selectNode}
            />
          ) : null}

          {activeLeftTab === "palette" ? <UiNodePalettePanel onAddNode={(kind) => openAddNode(activePath, kind)} /> : null}

          {activeLeftTab === "templates" ? (
            <UiTemplatePanel onAddTemplate={(template) => openAddTemplate(activePath, template)} />
          ) : null}
        </aside>

        <main className="ui-document-center">
          <UiDocumentPreviewPanel document={document} selectedNode={selectedNode} />
          <footer className="ui-document-breadcrumb">
            <span>UiDocument</span>
            {activePath.split(".").map((part, index, parts) => (
              <button key={`${part}:${index}`} type="button" onClick={() => selectNode(parts.slice(0, index + 1).join("."))}>
                {part}
              </button>
            ))}
          </footer>
        </main>

        <aside className="ui-document-right">
          <UiDocumentInspectorPanel
            busy={busy}
            canAddChild={canAddChild}
            canDuplicate={canDuplicate}
            canMoveDown={canMoveDown}
            canMoveUp={canMoveUp}
            canRemove={canRemove}
            document={document}
            selectedNode={selectedNode}
            onAddChild={() => openAddNode(activePath)}
            onDuplicate={() => void runNodeCommand()}
            onMoveDown={() => void runNodeCommand("down")}
            onMoveUp={() => void runNodeCommand("up")}
            onRemove={() => setConfirmRemovePath(activePath)}
          />
        </aside>
      </div>

      {pendingDialog?.kind === "add-document" ? (
        <AddUiDocumentDialog
          busy={busy}
          initialTemplate={pendingDialog.initialTemplate}
          onClose={() => setPendingDialog(null)}
          onCreate={handleCreateDocument}
        />
      ) : null}

      {pendingDialog?.kind === "add-node" ? (
        <AddUiNodeDialog
          busy={busy}
          initialKind={pendingDialog.initialKind}
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

      {confirmRemoveNodeValue ? (
        <ConfirmRemoveUiNodeDialog
          busy={busy}
          childCount={confirmRemoveNodeValue.childCount}
          nodeLabel={confirmRemoveNodeValue.label}
          nodePath={confirmRemoveNodeValue.path}
          onCancel={() => setConfirmRemovePath(null)}
          onConfirm={() => void confirmRemoveNode()}
        />
      ) : null}
    </section>
  );
}

function parentPathOf(nodePath: string): string {
  const parts = nodePath.split(".");
  if (parts.length <= 1) return nodePath;
  return parts.slice(0, -1).join(".");
}
