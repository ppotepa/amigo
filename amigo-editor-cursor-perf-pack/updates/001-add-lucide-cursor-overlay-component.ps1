param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_common.ps1")

$target = Join-Path $RepoRoot "crates/apps/amigo-editor/src/features/scenes/editor/SceneEditorCursorOverlay.tsx"

$content = @'
import {
  Ban,
  Hand,
  Maximize2,
  MousePointer2,
  Move,
  MoveHorizontal,
  MoveVertical,
  RotateCw,
} from "lucide-react";
import type { SceneEditorPoint, SceneEditorTool } from "./sceneEditorTypes";

function cursorIconForTool(tool: SceneEditorTool, pointerActive: boolean) {
  if (pointerActive) {
    if (tool === "pan") return <Hand size={22} strokeWidth={2.8} />;
    if (tool === "move") return <Move size={22} strokeWidth={2.8} />;
  }

  switch (tool) {
    case "select":
      return <MousePointer2 size={21} strokeWidth={2.6} />;
    case "move":
      return <Move size={22} strokeWidth={2.6} />;
    case "rotate":
      return <RotateCw size={22} strokeWidth={2.6} />;
    case "scale":
      return <Maximize2 size={22} strokeWidth={2.6} />;
    case "rect":
      return <Maximize2 size={22} strokeWidth={2.6} />;
    case "pan":
      return <Hand size={22} strokeWidth={2.6} />;
    default:
      return <MousePointer2 size={21} strokeWidth={2.6} />;
  }
}

export function SceneEditorCursorOverlay({
  framePoint,
  pointerActive,
  tool,
}: {
  framePoint: SceneEditorPoint | null;
  pointerActive: boolean;
  tool: SceneEditorTool;
}) {
  if (!framePoint) return null;

  return (
    <div
      className={`scene-editor-cursor-overlay scene-editor-cursor-overlay-${tool} ${
        pointerActive ? "is-active" : ""
      }`}
      aria-hidden
      style={{
        transform: `translate(${framePoint.x}px, ${framePoint.y}px)`,
      }}
    >
      <div className="scene-editor-cursor-shadow">
        {cursorIconForTool(tool, pointerActive)}
      </div>
      <div className="scene-editor-cursor-icon">
        {cursorIconForTool(tool, pointerActive)}
      </div>
      {tool === "move" && !pointerActive ? (
        <div className="scene-editor-cursor-axis-hint" aria-hidden>
          <MoveHorizontal size={13} strokeWidth={2.6} />
          <MoveVertical size={13} strokeWidth={2.6} />
        </div>
      ) : null}
      {tool === "rect" && pointerActive ? (
        <div className="scene-editor-cursor-blocked" aria-hidden>
          <Ban size={12} strokeWidth={2.6} />
        </div>
      ) : null}
    </div>
  );
}
'@

Write-NewFileIfChanged -Path $target -Content $content
