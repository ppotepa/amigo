import { useMemo, useState } from "react";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorUiTemplateKindDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
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
import { UiDocumentTreePanel } from "./UiDocumentTreePanel";
import { UiNodePalettePanel } from "./UiNodePalettePanel";
import { UiTemplatePanel } from "./UiTemplatePanel";
import { resolveUiDocumentEditorTarget } from "./uiDocumentTargetResolver";
import "./ui-document-editor.css";

type PendingDialog =
  | { kind: "add-node"; parentPath: string; initialKind?: UiNodeCreateKind }
  | { kind: "add-template"; parentPath: string; initialTemplate?: UiTemplateKind }
  | null;

export function UiDocumentStructureDock({
  instance,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const [activeTab, setActiveTab] = useState<UiDocumentEditorTab>("tree");
  const [pendingDialog, setPendingDialog] = useState<PendingDialog>(null);
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
  const selectedPath =
    services.selection?.kind === "uiNode" &&
    services.selection.nodeRef.entityId === document.entityId &&
    services.selection.nodeRef.componentIndex === document.componentIndex
      ? services.selection.nodeRef.nodePath
      : document.root.path;
  const activePath = findUiNode(document.root, selectedPath)?.path ?? document.root.path;

  function selectNode(nodePath: string) {
    services.selectUiNode?.({
      entityId: document.entityId,
      componentIndex: document.componentIndex,
      nodePath,
    });
  }

  function openAddNode(parentPath: string, initialKind?: UiNodeCreateKind) {
    selectNode(parentPath);
    setPendingDialog({ kind: "add-node", parentPath, initialKind });
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

    services.selectUiNode?.({
      entityId: selectedUiNode.entityId,
      componentIndex: selectedUiNode.componentIndex,
      nodePath: selectedUiNode.nodePath,
    });

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
          onSelectNode={selectNode}
        />
      ) : null}

      {activeTab === "palette" ? (
        <UiNodePalettePanel onAddNode={(kind) => openAddNode(activePath, kind)} />
      ) : null}

      {activeTab === "templates" ? (
        <UiTemplatePanel onAddTemplate={(template) => openAddTemplate(activePath, template)} />
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
