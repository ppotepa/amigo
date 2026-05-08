import type {
  EditorSceneComponentInstanceDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
} from "../../api/dto";
import type { EditorTargetRef } from "../editorTargetTypes";

// @codemap anchor:scene-target-adapter domain:scene-editor role:tree-adapter priority:P1 layer:app tags:editor-target,scene,selection
export function sceneToTarget(scene: EditorSceneSummaryDto): EditorTargetRef {
  return { kind: "scene", sceneId: scene.id };
}

export function sceneIdToTarget(sceneId: string): EditorTargetRef {
  return { kind: "scene", sceneId };
}

export function sceneEntityToTarget({
  sceneId,
  entity,
}: {
  sceneId: string;
  entity: EditorSceneEntityDto;
}): EditorTargetRef {
  return {
    kind: "sceneEntity",
    sceneId,
    entityId: entity.id,
  };
}

export function sceneEntityIdToTarget(sceneId: string, entityId: string): EditorTargetRef {
  return {
    kind: "sceneEntity",
    sceneId,
    entityId,
  };
}

// @codemap anchor:scene-component-target-adapter domain:scene-editor role:tree-adapter priority:P1 layer:app tags:editor-target,component,metadata
export function sceneComponentToTarget({
  sceneId,
  entityId,
  component,
}: {
  sceneId: string;
  entityId: string;
  component: EditorSceneComponentInstanceDto;
}): EditorTargetRef {
  return {
    kind: "component",
    sceneId,
    entityId,
    componentIndex: component.componentIndex,
    componentType: component.typeName,
  };
}

export function sceneComponentIndexToTarget({
  sceneId,
  entityId,
  componentIndex,
  componentType,
}: {
  sceneId: string;
  entityId: string;
  componentIndex: number;
  componentType: string;
}): EditorTargetRef {
  return {
    kind: "component",
    sceneId,
    entityId,
    componentIndex,
    componentType,
  };
}
