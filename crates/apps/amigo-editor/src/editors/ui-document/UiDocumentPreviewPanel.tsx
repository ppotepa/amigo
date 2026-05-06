import type { CSSProperties } from "react";
import { useState } from "react";
import { Bug, LayoutGrid, Monitor } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";
import { UiDocumentPreviewRenderer } from "./UiDocumentPreviewRenderer";
import {
  DEFAULT_UI_ARTBOARD_HEIGHT,
  DEFAULT_UI_ARTBOARD_WIDTH,
  type UiPreviewMode,
} from "./uiPreviewRenderModel";

const ZOOM_OPTIONS = [0.5, 0.75, 1] as const;

export function UiDocumentPreviewPanel({
  document,
  selectedNode,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  selectedNode: EditorUiNodeDto | null;
  onSelectNode: (nodePath: string) => void;
}) {
  const [mode, setMode] = useState<UiPreviewMode>("layout");
  const [zoom, setZoom] = useState<(typeof ZOOM_OPTIONS)[number]>(0.75);

  return (
    <section className="ui-document-preview-panel">
      <header>
        <div>
          <h3>Preview</h3>
          <span>
            {document.targetLayer ?? "screen-space"} / {DEFAULT_UI_ARTBOARD_WIDTH}x{DEFAULT_UI_ARTBOARD_HEIGHT}
          </span>
        </div>

        <div className="ui-preview-toolbar">
          <div className="ui-preview-mode-switch" role="group" aria-label="Preview mode">
            <button
              className={mode === "preview" ? "active" : ""}
              type="button"
              onClick={() => setMode("preview")}
            >
              <Monitor size={13} />
              Preview
            </button>
            <button
              className={mode === "layout" ? "active" : ""}
              type="button"
              onClick={() => setMode("layout")}
            >
              <LayoutGrid size={13} />
              Layout
            </button>
            <button
              className={mode === "debug" ? "active" : ""}
              type="button"
              onClick={() => setMode("debug")}
            >
              <Bug size={13} />
              Debug
            </button>
          </div>

          <select
            className="ui-preview-zoom-select"
            value={zoom}
            onChange={(event) => setZoom(Number(event.target.value) as (typeof ZOOM_OPTIONS)[number])}
          >
            {ZOOM_OPTIONS.map((option) => (
              <option key={option} value={option}>
                {Math.round(option * 100)}%
              </option>
            ))}
          </select>
        </div>
      </header>

      <div className="ui-preview-canvas">
        <div
          className={`ui-preview-artboard ui-preview-artboard-${mode}`}
          style={
            {
              "--ui-preview-zoom": zoom,
              "--ui-preview-artboard-width": `${DEFAULT_UI_ARTBOARD_WIDTH}px`,
              "--ui-preview-artboard-height": `${DEFAULT_UI_ARTBOARD_HEIGHT}px`,
            } as CSSProperties
          }
        >
          <UiDocumentPreviewRenderer
            document={document}
            mode={mode}
            selectedPath={selectedNode?.path ?? document.root.path}
            onSelectNode={onSelectNode}
          />
        </div>
      </div>

      <footer className="ui-preview-statusbar">
        <span>
          Selected: <code>{selectedNode?.path ?? document.root.path}</code>
        </span>
        <span>{document.entityName}</span>
      </footer>
    </section>
  );
}
