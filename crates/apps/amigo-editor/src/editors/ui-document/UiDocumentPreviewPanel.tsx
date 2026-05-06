import { useState } from "react";
import { Bug, LayoutGrid, Monitor } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";
import {
  PreviewArtboard,
  PreviewCanvasSurface,
  PreviewStatusBar,
  PreviewViewControls,
} from "../../ui/preview";
import { UiDocumentPreviewRenderer } from "./UiDocumentPreviewRenderer";
import {
  DEFAULT_UI_ARTBOARD_HEIGHT,
  DEFAULT_UI_ARTBOARD_WIDTH,
  type UiPreviewMode,
} from "./uiPreviewRenderModel";

const ZOOM_OPTIONS = [0.5, 0.75, 1] as const;
type PreviewZoom = (typeof ZOOM_OPTIONS)[number];

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
  const [zoom, setZoom] = useState<PreviewZoom>(0.75);
  const selectedPath = selectedNode?.path ?? document.root.path;

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

          <PreviewViewControls
            zoom={zoom}
            zoomOptions={ZOOM_OPTIONS}
            onZoomChange={(nextZoom) => setZoom(nextZoom as PreviewZoom)}
          />
        </div>
      </header>

      <PreviewCanvasSurface className="ui-preview-canvas">
        <PreviewArtboard
          className={`ui-preview-artboard ui-preview-artboard-${mode} ${
            mode === "layout" || mode === "debug" ? "preview-artboard-grid" : ""
          }`}
          width={DEFAULT_UI_ARTBOARD_WIDTH}
          height={DEFAULT_UI_ARTBOARD_HEIGHT}
          zoom={zoom}
        >
          <UiDocumentPreviewRenderer
            document={document}
            mode={mode}
            selectedPath={selectedPath}
            onSelectNode={onSelectNode}
          />
        </PreviewArtboard>
      </PreviewCanvasSurface>

      <PreviewStatusBar
        left={
          <>
            Selected: <code>{selectedPath}</code>
          </>
        }
        right={document.entityName}
      />
    </section>
  );
}
