import { useState } from "react";
import { Play, RefreshCw } from "lucide-react";
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

// @codemap anchor:ui-document-preview-panel domain:ui-document role:preview priority:P1 layer:app tags:simple,realtime,artboard
export function UiDocumentPreviewPanel({
  document,
  focusPath,
  selectedNode,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  focusPath?: string | null;
  selectedNode: EditorUiNodeDto | null;
  onSelectNode: (nodePath: string) => void;
}) {
  const [mode, setMode] = useState<UiPreviewMode>("simple");
  const [zoom, setZoom] = useState<PreviewZoom>(0.75);
  const [revision, setRevision] = useState(0);
  const selectedPath = selectedNode?.path ?? document.root.path;

  return (
    <section className="ui-document-preview-panel">
      <header>
        <div>
          <h3>UI / main-menu.ui</h3>
          <span>
            {document.targetLayer ?? "screen-space"} / {DEFAULT_UI_ARTBOARD_WIDTH}x{DEFAULT_UI_ARTBOARD_HEIGHT}
          </span>
        </div>

        <div className="ui-preview-toolbar">
          <div className="ui-preview-mode-switch" role="group" aria-label="UI preview mode">
            <button
              className={mode === "simple" ? "active" : ""}
              type="button"
              onClick={() => setMode("simple")}
            >
              Simple
            </button>
            <button
              className={mode === "realtime" ? "active" : ""}
              type="button"
              onClick={() => setMode("realtime")}
            >
              Realtime
            </button>
          </div>

          {mode === "realtime" ? (
            <>
              <button className="button button-ghost" type="button" title="Play runtime preview">
                <Play size={13} />
                Play
              </button>
              <button
                className="button button-ghost"
                type="button"
                title="Refresh runtime preview"
                onClick={() => setRevision((value) => value + 1)}
              >
                <RefreshCw size={13} />
                Refresh Preview
              </button>
            </>
          ) : null}

          <PreviewViewControls
            zoom={zoom}
            zoomOptions={ZOOM_OPTIONS}
            onZoomChange={(nextZoom) => setZoom(nextZoom as PreviewZoom)}
          />
        </div>
      </header>

      <PreviewCanvasSurface className={`ui-preview-canvas ui-preview-canvas-${mode}`}>
        <PreviewArtboard
          className={`ui-preview-artboard ui-preview-artboard-${mode}`}
          width={DEFAULT_UI_ARTBOARD_WIDTH}
          height={DEFAULT_UI_ARTBOARD_HEIGHT}
          zoom={zoom}
        >
          <UiDocumentPreviewRenderer
            document={document}
            focusPath={focusPath ?? null}
            mode={mode}
            selectedPath={selectedPath}
            revision={revision}
            onSelectNode={onSelectNode}
          />
        </PreviewArtboard>
      </PreviewCanvasSurface>

      <PreviewStatusBar
        left={
          <>
            Focus: <code>{focusPath ?? "document"}</code> / Selected: <code>{selectedPath}</code>
          </>
        }
        right={mode === "simple" ? "Simple View" : "Realtime View"}
      />
    </section>
  );
}
