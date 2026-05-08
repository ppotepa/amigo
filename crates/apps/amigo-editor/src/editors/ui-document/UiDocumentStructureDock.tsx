import { useMemo, useState } from "react";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorUiNodeMoveDirectionDto,
  EditorUiTemplateKindDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import { uiNodeToTarget } from "../../editor-targets/adapters/uiDocumentTargetAdapter";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { findUiNode } from "./uiDocumentEditorModel";
import type {
  AddUiNodeDraft,
  AddUiTemplateDraft,
  UiDocumentEditorTab,
  UiNodeCreateKind,
  UiTemplateKind,
} from "./uiDocumentEditorTypes";
import { AddUiNodeDialog } from "./AddUiNodeDialog";
import { AddUiTemplateDialog } from "./AddUiTemplateDialog";
import { ConfirmRemoveUiNodeDialog } from "./ConfirmRemoveUiNodeDialog";
import { getSiblingInfo } from "./uiDocumentEditorModel";
import { firstAllowedUiChildKind, uiNodeCanAddChild } from "./uiNodeCapabilities";
import { UiNodeActionsPanel } from "./UiNodeActionsPanel";
import { UiDocumentTreePanel } from "./UiDocumentTreePanel";
import { UiNodePalettePanel } from "./UiNodePalettePanel";
import { UiTemplatePanel } from "./UiTemplatePanel";
import { resolveUiDocumentEditorTarget } from "./uiDocumentTargetResolver";
import "./ui-document-editor.css";

type PendingDialog =
  | { kind: "add-node"; parentPath: string; initialKind?: UiNodeCreateKind }
  | { kind: "add-template"; parentPath: string; initialTemplate?: UiTemplateKind }
  | null;

// @codemap anchor:ui-document-structure-dock domain:ui-document role:tree priority:P1 layer:app tags:ui-document,dock,structure
export function UiDocumentStructureDock({
  instance,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const [activeTab, setActiveTab] = useState<UiDocumentEditorTab>("tree");
  const [pendingDialog, setPendingDialog] = useState<PendingDialog>(null);
  const [confirmRemovePath, setConfirmRemovePath] = useState<string | null>(null);
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

  if (targetResolution.kind !== "resolved") {
    return (
      <section className="workspace-section">
        <h3>UI Structure</h3>
        <p className="muted workspace-note">No active UiDocument.</p>
      </section>
    );
  }

  const { document, target } = targetResolution;
  const currentSelection = services.currentEditorTarget?.selection ?? services.selection;
  const selectedPath =
    currentSelection?.kind === "uiNode" &&
    currentSelection.nodeRef.entityId === document.entityId &&
    currentSelection.nodeRef.componentIndex === document.componentIndex
      ? currentSelection.nodeRef.nodePath
      : document.root.path;
  const activePath = findUiNode(document.root, selectedPath)?.path ?? document.root.path;
  const selectedNode = findUiNode(document.root, activePath);
  const siblingInfo = getSiblingInfo(document.root, activePath);
  const canAddChild = uiNodeCanAddChild(selectedNode);
  const canDuplicate = Boolean(selectedNode && selectedNode.path !== document.root.path);
  const canRemove = canDuplicate;
  const canMoveUp = Boolean(siblingInfo && siblingInfo.index > 0);
  const canMoveDown = Boolean(siblingInfo && siblingInfo.index + 1 < siblingInfo.count);
  const confirmRemoveNodeValue = confirmRemovePath ? findUiNode(document.root, confirmRemovePath) : null;

  function uiNodeTarget(nodePath: string) {
    return uiNodeToTarget({
      sceneId: target.sceneId,
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath,
    });
  }

  function selectNode(nodePath: string) {
    services.activateEditorTarget?.(uiNodeTarget(nodePath), "select");
  }

  function openNodeScopedView(nodePath: string) {
    services.activateEditorTarget?.(uiNodeTarget(nodePath), "open");
  }

  function openAddNode(parentPath: string, initialKind?: UiNodeCreateKind) {
    const parentNode = findUiNode(document.root, parentPath);
    if (!uiNodeCanAddChild(parentNode)) {
      setError(parentNode ? `${parentNode.label} cannot contain child nodes.` : "Select a valid parent node.");
      return;
    }

    selectNode(parentPath);
    setError(null);
    setPendingDialog({
      kind: "add-node",
      parentPath,
      initialKind: initialKind ?? firstAllowedUiChildKind(parentNode) ?? undefined,
    });
  }

  function openAddTemplate(parentPath: string, initialTemplate?: UiTemplateKind) {
    selectNode(parentPath);
    setPendingDialog({ kind: "add-template", parentPath, initialTemplate });
  }

  async function runUiCommand(
    command: EditorCommandDto,
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

      await services.refreshEditorSnapshot?.();
      await services.refreshSceneHierarchy?.();
      setPendingDialog(null);
      setConfirmRemovePath(null);
      selectUiNodeFromCommandResult(result);
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

    services.activateEditorTarget?.(
      uiNodeToTarget({
        sceneId: target.sceneId,
        entityId: selectedUiNode.entityId,
        componentIndex: selectedUiNode.componentIndex,
        nodePath: selectedUiNode.nodePath,
      }),
      "select",
    );

    return true;
  }

  async function handleCreateNode(draft: AddUiNodeDraft) {
    await runUiCommand({
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
    });
  }

  async function handleCreateTemplate(draft: AddUiTemplateDraft) {
    await runUiCommand({
      type: "AddUiTemplate",
      sceneId: target.sceneId,
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      parentPath: draft.parentPath,
      template: supportedTemplate(draft.template),
      idPrefix: draft.idPrefix,
      insertIndex: null,
    });
  }

  async function runNodeCommand(direction?: EditorUiNodeMoveDirectionDto) {
    if (!selectedNode) return;

    if (direction) {
      await runUiCommand({
        type: "MoveUiNode",
        sceneId: target.sceneId,
        entityId: document.entityId,
        componentIndex: document.componentIndex,
        nodePath: selectedNode.path,
        direction,
      });
      return;
    }

    await runUiCommand({
      type: "DuplicateUiNode",
      sceneId: target.sceneId,
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath: selectedNode.path,
      newId: null,
      copyActions: false,
    });
  }

  async function confirmRemoveNode() {
    if (!confirmRemovePath) return;
    const removedPath = confirmRemovePath;
    await runUiCommand({
      type: "RemoveUiNode",
      sceneId: target.sceneId,
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath: removedPath,
    });
  }

  return (
    <div className="dock-scroll">
      {error ? (
        <section className="workspace-section">
          <p className="ui-dialog-error">{error}</p>
        </section>
      ) : null}

      <nav className="ui-document-left-tabs">
        <button className={activeTab === "tree" ? "active" : ""} type="button" onClick={() => setActiveTab("tree")}>
          Tree
        </button>
        <button className={activeTab === "palette" ? "active" : ""} type="button" onClick={() => setActiveTab("palette")}>
          Palette
        </button>
        <button className={activeTab === "templates" ? "active" : ""} type="button" onClick={() => setActiveTab("templates")}>
          Templates
        </button>
      </nav>

      {activeTab === "tree" ? (
        <UiDocumentTreePanel
          document={document}
          selectedPath={activePath}
          onAddChild={(parentPath) => openAddNode(parentPath)}
          onOpenNode={openNodeScopedView}
          onSelectNode={selectNode}
        />
      ) : null}

      {activeTab === "palette" ? (
        <UiNodePalettePanel
          parentNode={selectedNode}
          onAddNode={(kind) => openAddNode(activePath, kind)}
        />
      ) : null}

      {activeTab === "templates" ? (
        <UiTemplatePanel onAddTemplate={(template) => openAddTemplate(activePath, template)} />
      ) : null}

      <UiNodeActionsPanel
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

      {pendingDialog?.kind === "add-node" ? (
        <AddUiNodeDialog
          busy={busy}
          initialKind={pendingDialog.initialKind}
          parentNode={findUiNode(document.root, pendingDialog.parentPath)}
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
    </div>
  );
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
