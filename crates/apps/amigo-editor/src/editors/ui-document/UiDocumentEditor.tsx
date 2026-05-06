import { useMemo, useState } from "react";
import { AlertTriangle, Boxes, FilePlus2, Plus } from "lucide-react";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { findUiDocument, findUiNode } from "./uiDocumentEditorModel";
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
import { UiDocumentInspectorPanel } from "./UiDocumentInspectorPanel";
import { UiDocumentPreviewPanel } from "./UiDocumentPreviewPanel";
import { UiDocumentStartScreen } from "./UiDocumentStartScreen";
import { UiDocumentTreePanel } from "./UiDocumentTreePanel";
import { UiNodePalettePanel } from "./UiNodePalettePanel";
import { UiTemplatePanel } from "./UiTemplatePanel";
import "./ui-document-editor.css";

type PendingDialog =
  | { kind: "add-document" }
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

  const target = useMemo(() => {
    const context = instance.context ?? {};
    if (!context.sceneId || !context.entityId || context.componentIndex == null) {
      return null;
    }

    return {
      modId: services.details?.id ?? "",
      sceneId: context.sceneId,
      entityId: context.entityId,
      componentIndex: Number(context.componentIndex),
    };
  }, [instance.context, services.details?.id]);

  const document = findUiDocument(services.hierarchy ?? null, target);
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

  function openAddNode(parentPath: string, initialKind?: UiNodeCreateKind) {
    setPendingDialog({ kind: "add-node", parentPath, initialKind });
  }

  function openAddTemplate(parentPath: string, initialTemplate?: UiTemplateKind) {
    setPendingDialog({ kind: "add-template", parentPath, initialTemplate });
  }

  function handleCreateDocument(draft: AddUiDocumentDraft) {
    setNotice(`Create UI Document is planned. Draft: ${draft.name} / ${draft.template}`);
    setPendingDialog(null);
  }

  function handleCreateNode(draft: AddUiNodeDraft) {
    setNotice(`Add Node is planned. Draft: ${draft.kind} ${draft.id} under ${draft.parentPath}`);
    setPendingDialog(null);
  }

  function handleCreateTemplate(draft: AddUiTemplateDraft) {
    setNotice(`Add Template is planned. Draft: ${draft.template} under ${draft.parentPath}`);
    setPendingDialog(null);
  }

  if (!document) {
    return (
      <>
        <UiDocumentStartScreen onCreateDocument={() => setPendingDialog({ kind: "add-document" })} />

        {pendingDialog?.kind === "add-document" ? (
          <AddUiDocumentDialog onClose={() => setPendingDialog(null)} onCreate={handleCreateDocument} />
        ) : null}
      </>
    );
  }

  const activePath = selectedNode?.path ?? document.root.path;

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
          <button className="button button-ghost" type="button" onClick={() => setPendingDialog({ kind: "add-document" })}>
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
            document={document}
            selectedNode={selectedNode}
            onAddChild={() => openAddNode(activePath)}
            onDuplicate={() => setNotice("Duplicate Node command comes next.")}
            onMoveDown={() => setNotice("Move Down command comes next.")}
            onMoveUp={() => setNotice("Move Up command comes next.")}
            onRemove={() => setNotice("Remove Node command comes next.")}
          />
        </aside>
      </div>

      {pendingDialog?.kind === "add-document" ? (
        <AddUiDocumentDialog onClose={() => setPendingDialog(null)} onCreate={handleCreateDocument} />
      ) : null}

      {pendingDialog?.kind === "add-node" ? (
        <AddUiNodeDialog
          initialKind={pendingDialog.initialKind}
          parentPath={pendingDialog.parentPath}
          onClose={() => setPendingDialog(null)}
          onCreate={handleCreateNode}
        />
      ) : null}

      {pendingDialog?.kind === "add-template" ? (
        <AddUiTemplateDialog
          initialTemplate={pendingDialog.initialTemplate}
          parentPath={pendingDialog.parentPath}
          onClose={() => setPendingDialog(null)}
          onCreate={handleCreateTemplate}
        />
      ) : null}
    </section>
  );
}
