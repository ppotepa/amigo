import type { SceneEditorCommand } from "./sceneEditorTypes";

export function describeSceneEditorCommand(command: SceneEditorCommand): string {
  switch (command.type) {
    case "selectEntity":
      return command.entityId ? `Select entity ${command.entityId}` : "Clear scene selection";
    case "moveEntity":
      return `Move entity ${command.entityId} to ${Math.round(command.x)}, ${Math.round(command.y)}`;
  }
}

export function emitSceneEditorCommand(command: SceneEditorCommand): void {
  console.info("[SceneEditor]", describeSceneEditorCommand(command), command);
}
