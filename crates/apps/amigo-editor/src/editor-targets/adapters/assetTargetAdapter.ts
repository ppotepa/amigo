import type { ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import type { AssetTreeNode } from "../../assets/assetTreeBuilder";
import type { EditorTargetRef } from "../editorTargetTypes";
import { projectFilePathToTarget } from "./fileTargetAdapter";

// @codemap anchor:asset-target-adapter domain:assets role:tree-adapter priority:P1 layer:app tags:editor-target,assets,selection
export function assetToTarget(asset: ManagedAssetDto): EditorTargetRef {
  return { kind: "asset", assetKey: asset.assetKey };
}

export function assetKeyToTarget(assetKey: string): EditorTargetRef {
  return { kind: "asset", assetKey };
}

export function rawAssetToTarget(file: RawAssetFileDto): EditorTargetRef {
  return projectFilePathToTarget(file.relativePath);
}

export function assetTreeNodeToTarget(node: AssetTreeNode): EditorTargetRef | null {
  if (node.asset) return assetToTarget(node.asset);
  if (node.rawFile) return rawAssetToTarget(node.rawFile);
  return null;
}
