import {
  AlertTriangle,
  Box,
  Cuboid,
  Image,
  Layers3,
  MousePointer2,
  Move,
  Play,
  RotateCw,
  Scaling,
  Square,
  SquareDashedMousePointer,
} from "lucide-react";
import type React from "react";
import { DebugSourceLabel } from "../../../debug/debugSource";
import type {
  EditorFrameDto,
  EditorModeSessionDto,
  EditorSceneSnapshotQualityDto,
} from "../../../api/dto";
import type { SceneEditorPreviewSyncState } from "./sceneEditorPreviewSync";
import type {
  SceneEditorCanvasKind,
  SceneEditorMode,
  SceneEditorPoint,
  SceneEditorTool,
} from "./sceneEditorTypes";
import { zoomPercent } from "./sceneEditorTransforms";

export function SceneEditorStatusBar({
  canvasKind,
  frame,
  mode,
  mouseScenePoint,
  previewSync,
  quality,
  selectedEntityName,
  session,
  tool,
  zoom,
}: {
  canvasKind: SceneEditorCanvasKind;
  frame?: EditorFrameDto | null;
  mode: SceneEditorMode;
  mouseScenePoint?: SceneEditorPoint | null;
  previewSync?: SceneEditorPreviewSyncState;
  quality?: EditorSceneSnapshotQualityDto;
  selectedEntityName?: string | null;
  session?: EditorModeSessionDto | null;
  tool: SceneEditorTool;
  zoom: number;
}) {
  const diagnosticsCount = quality
    ? Object.values(quality.diagnosticsByCode).reduce((total, count) => total + count, 0)
    : 0;

  return (
    <footer className="scene-editor-status-bar" data-editor-chrome="true">
      <DebugSourceLabel source="src/features/scenes/editor/SceneEditorStatusBar.tsx" />
      <StatusChip icon={canvasKindIcon(canvasKind)} label={canvasKindLabel(canvasKind)} title="Canvas kind" />
      <StatusChip icon={modeIcon(mode)} label={modeLabel(mode)} title="Interaction mode" />
      <StatusChip icon={toolIcon(tool)} label={toolLabel(tool)} title="Active tool" />

      <span className="scene-editor-status-separator" />

      <StatusChip
        label={mouseScenePoint ? `x ${Math.round(mouseScenePoint.x)} y ${Math.round(mouseScenePoint.y)}` : "x - y -"}
        title="Cursor scene position"
        wide
      />
      <StatusChip
        label={selectedEntityName ? `Selected: ${selectedEntityName}` : "No selection"}
        title="Selected entity"
        wide
      />

      <span className="scene-editor-status-spacer" />

      {session ? <StatusChip label={`Engine editor · rev ${session.revision}`} title="Editor session revision" /> : null}
      {frame ? (
        <StatusChip
          icon={<Image size={12} />}
          label={frame.transport}
          title="Editor frame transport"
        />
      ) : null}
      <StatusChip label={zoomPercent(zoom)} title="Viewport zoom" />
      {previewSync ? <PreviewSyncChip previewSync={previewSync} /> : null}
      <DiagnosticsChip count={diagnosticsCount} />
    </footer>
  );
}

function StatusChip({
  icon,
  label,
  title,
  wide = false,
}: {
  icon?: React.ReactNode;
  label: string;
  title?: string;
  wide?: boolean;
}) {
  return (
    <span className={`scene-editor-status-chip ${wide ? "wide" : ""}`} title={title}>
      {icon ? <span className="scene-editor-status-icon">{icon}</span> : null}
      <span>{label}</span>
    </span>
  );
}

function PreviewSyncChip({ previewSync }: { previewSync: SceneEditorPreviewSyncState }) {
  return (
    <span
      className={`scene-editor-status-chip scene-editor-status-sync scene-editor-status-sync-${previewSync.status}`}
      title={previewSync.message ?? `Preview ${previewSync.status}`}
    >
      <span className="scene-editor-status-dot" />
      <span>{previewSync.status}</span>
    </span>
  );
}

function DiagnosticsChip({ count }: { count: number }) {
  return (
    <span
      className={`scene-editor-status-chip scene-editor-status-diagnostics ${count > 0 ? "has-diagnostics" : ""}`}
      title={count > 0 ? `${count} editor diagnostics` : "No editor diagnostics"}
    >
      <span className="scene-editor-status-icon">
        <AlertTriangle size={12} />
      </span>
      <span>{count}</span>
    </span>
  );
}

function canvasKindIcon(kind: SceneEditorCanvasKind) {
  if (kind === "3d") return <Box size={12} />;
  if (kind === "2.5d") return <Layers3 size={12} />;
  return <Square size={12} />;
}

function canvasKindLabel(kind: SceneEditorCanvasKind): string {
  if (kind === "3d") return "3D";
  if (kind === "2.5d") return "2.5D";
  return "2D";
}

function modeIcon(mode: SceneEditorMode) {
  if (mode === "play") return <Play size={12} />;
  if (mode === "preview") return <MousePointer2 size={12} />;
  return <SquareDashedMousePointer size={12} />;
}

function modeLabel(mode: SceneEditorMode): string {
  if (mode === "play") return "Play";
  if (mode === "preview") return "Preview";
  return "Edit";
}

function toolIcon(tool: SceneEditorTool) {
  if (tool === "move") return <Move size={12} />;
  if (tool === "rotate") return <RotateCw size={12} />;
  if (tool === "scale") return <Scaling size={12} />;
  if (tool === "rect") return <Cuboid size={12} />;
  if (tool === "pan") return <Move size={12} />;
  return <MousePointer2 size={12} />;
}

function toolLabel(tool: SceneEditorTool): string {
  if (tool === "move") return "Move";
  if (tool === "rotate") return "Rotate";
  if (tool === "scale") return "Scale";
  if (tool === "rect") return "Rect";
  if (tool === "pan") return "Pan";
  return "Select";
}
