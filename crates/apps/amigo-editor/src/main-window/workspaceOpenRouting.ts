import type {
  EditorModDetailsDto,
  EditorProjectFileDto,
  EditorProjectTreeDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
} from "../api/dto";
import { projectFileFromManagedAsset } from "../assets/assetProjectFiles";
import { findProjectFile, normalizePath } from "../features/files/fileTreeSelectors";
import { resolveFileWorkspaceDescriptor } from "../features/files/fileWorkspaceRules";
import type { OpenWorkspaceEditorRequest } from "./workspaceOpenTypes";

export function sceneForManagedAsset(
  details: EditorModDetailsDto | null,
  asset: ManagedAssetDto,
): EditorSceneSummaryDto | null {
  if (!details || asset.kind !== "scene") return null;

  const descriptorPath = normalizePath(asset.descriptorRelativePath);
  const absoluteDescriptorPath = normalizePath(asset.descriptorPath);
  const assetKey = normalizePath(asset.assetKey);
  const assetId = normalizePath(asset.assetId);

  return details.scenes.find((scene) => {
    const sceneDocumentPath = normalizePath(scene.documentPath);
    const scenePath = normalizePath(scene.path);
    const sceneAssetKeys = [
      scenePath,
      sceneDocumentPath,
      `${scenePath}/scene.yml`,
      `${scenePath}/scene.yaml`,
      `scenes/${scene.id}`,
      `scenes/${scene.id}/scene.yml`,
      `scenes/${scene.id}/scene.yaml`,
    ].map(normalizePath);

    return sceneAssetKeys.some((key) => (
      descriptorPath === key ||
      absoluteDescriptorPath.endsWith(key) ||
      sceneDocumentPath.endsWith(descriptorPath) ||
      descriptorPath.endsWith(sceneDocumentPath) ||
      assetId === scene.id ||
      assetKey.endsWith(key)
    ));
  }) ?? null;
}

export function resolveManagedAssetOpenRequest({
  asset,
  details,
  projectTree,
}: {
  asset: ManagedAssetDto;
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
}): OpenWorkspaceEditorRequest {
  const scene = sceneForManagedAsset(details, asset);
  if (scene) {
    return { kind: "scene", scene };
  }

  const projectFile = projectFileForManagedAsset(asset, projectTree);
  if (projectFile) {
    return { kind: "project-file", file: projectFile };
  }

  return { kind: "asset", asset };
}

export function projectFileForManagedAsset(
  asset: ManagedAssetDto,
  projectTree?: EditorProjectTreeDto,
): EditorProjectFileDto | null {
  if (!projectTree) {
    return projectFileFromManagedAsset(asset);
  }

  const descriptorRelativePath = normalizePath(asset.descriptorRelativePath);
  const fallbackFile = projectFileFromManagedAsset(asset);

  return (
    findProjectFile(projectTree.root, descriptorRelativePath) ??
    findProjectFile(projectTree.root, fallbackFile.relativePath) ??
    fallbackFile
  );
}

export function componentOpenRequestForProjectFile(
  file: EditorProjectFileDto,
): OpenWorkspaceEditorRequest {
  const descriptor = resolveFileWorkspaceDescriptor(file);

  return {
    kind: "component",
    componentId: descriptor.componentId,
    resourceUri: file.relativePath,
    titleOverride: file.name,
    context: {
      fileKind: descriptor.fileKind ?? file.kind,
      filePath: file.relativePath,
    },
  };
}
