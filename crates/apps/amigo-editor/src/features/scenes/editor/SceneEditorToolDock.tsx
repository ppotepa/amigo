import type React from "react";
import {
  Hand,
  Maximize2,
  MousePointer2,
  Move,
  RotateCw,
  Scaling,
} from "lucide-react";
import type { SceneEditorTool } from "./sceneEditorTypes";

const TOOL_DOCK: Array<{
  id: SceneEditorTool;
  label: string;
  icon: React.ReactNode;
}> = [
  { id: "select", label: "Select", icon: <MousePointer2 size={15} /> },
  { id: "move", label: "Move", icon: <Move size={15} /> },
  { id: "scale", label: "Scale", icon: <Scaling size={15} /> },
  { id: "rotate", label: "Rotate", icon: <RotateCw size={15} /> },
  { id: "rect", label: "Rect", icon: <Maximize2 size={15} /> },
  { id: "pan", label: "Pan", icon: <Hand size={15} /> },
];

export function SceneEditorToolDock({
  activeTool,
  onToolChange,
}: {
  activeTool: SceneEditorTool;
  onToolChange: (tool: SceneEditorTool) => void;
}) {
  return (
    <div
      className="scene-editor-floating-dock scene-editor-tool-dock"
      aria-label="Transform tools"
      data-editor-chrome="true"
      onPointerDown={(event) => event.stopPropagation()}
      onPointerMove={(event) => event.stopPropagation()}
      onPointerUp={(event) => event.stopPropagation()}
      onClick={(event) => event.stopPropagation()}
    >
      {TOOL_DOCK.map((entry) => (
        <button
          key={entry.id}
          className={`scene-editor-floating-button ${activeTool === entry.id ? "selected" : ""}`}
          type="button"
          title={entry.label}
          onPointerDown={(event) => event.stopPropagation()}
          onClick={(event) => {
            event.stopPropagation();
            onToolChange(entry.id);
          }}
        >
          {entry.icon}
        </button>
      ))}
    </div>
  );
}
