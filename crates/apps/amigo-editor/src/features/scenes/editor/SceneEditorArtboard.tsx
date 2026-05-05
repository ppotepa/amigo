import type React from "react";
import type { ScenePreviewDto } from "../../../api/dto";
import type {
  SceneEditorEntity,
  SceneEditorMode,
  SceneEditorResolution,
  SceneEditorTool,
  SceneEditorViewportState,
} from "./sceneEditorTypes";
import { SceneEditorRenderLayer } from "./SceneEditorRenderLayer";
import { SceneEditorOverlay } from "./SceneEditorOverlay";

export function SceneEditorArtboard({
  entities,
  mode,
  onBeginEntityDrag,
  onSelectEntity,
  preview,
  resolution,
  selectedEntityId,
  tool,
  viewport,
}: {
  preview?: ScenePreviewDto;
  resolution: SceneEditorResolution;
  entities: SceneEditorEntity[];
  selectedEntityId: string | null;
  viewport: SceneEditorViewportState;
  mode: SceneEditorMode;
  tool: SceneEditorTool;
  onSelectEntity: (entityId: string | null) => void;
  onBeginEntityDrag: (event: React.PointerEvent<HTMLButtonElement>, entity: SceneEditorEntity) => void;
}) {
  return (
    <div
      className="scene-editor-artboard"
      style={{
        width: resolution.width,
        height: resolution.height,
        transform: `translate(${viewport.panX}px, ${viewport.panY}px) scale(${viewport.zoom})`,
      }}
      onClick={(event) => {
        event.stopPropagation();
        onSelectEntity(null);
      }}
    >
      <SceneEditorRenderLayer preview={preview} />
      <SceneEditorOverlay
        entities={entities}
        mode={mode}
        selectedEntityId={selectedEntityId}
        tool={tool}
        onSelectEntity={onSelectEntity}
        onBeginEntityDrag={onBeginEntityDrag}
      />
    </div>
  );
}
