import { describe, expect, it } from "vitest";
import type { EditorProjectFileDto } from "../../api/dto";
import {
  assetDisplayLabel,
  mergeProjectTrees,
  normalizeProjectTreeNode,
  projectNodeKindLabel,
  projectNodeMatchesSearch,
  relativeProjectPath,
  statusForEditorStatus,
  type ProjectTreeNode,
} from "./projectTreeModel";

describe("projectTreeModel", () => {
  it("normalizes structure nodes into project tree nodes", () => {
    const node = normalizeProjectTreeNode({
      id: "ghost:scenes",
      label: "scenes",
      kind: "expectedFolder",
      icon: "Folder",
      status: null,
      count: null,
      path: null,
      expectedPath: "scenes/",
      exists: false,
      empty: false,
      ghost: true,
      children: [],
    });

    expect(node).toMatchObject({
      id: "ghost:scenes",
      kind: "expectedFolder",
      expectedPath: "scenes/",
      exists: false,
      ghost: true,
      empty: false,
    });
  });

  it("merges preferred children onto fallback nodes by id", () => {
    const fallback: ProjectTreeNode = {
      id: "root",
      label: "Root",
      kind: "modRoot",
      icon: "Mod",
      exists: true,
      children: [
        {
          id: "overview",
          label: "Overview",
          kind: "overview",
          icon: "Info",
          exists: true,
        },
      ],
    };

    const merged = mergeProjectTrees({
      id: "root",
      label: "Root",
      kind: "modRoot",
      icon: "Mod",
      exists: true,
      children: [
        {
          id: "overview",
          label: "Overview",
          kind: "overview",
          icon: "Info",
          exists: true,
          status: "warn",
          children: [],
        },
      ],
    }, fallback);

    expect(merged.children?.[0]).toMatchObject({
      id: "overview",
      status: "warn",
      kind: "overview",
    });
  });

  it("keeps semantic scene nodes flat when structure tree includes descriptor children", () => {
    const fallback: ProjectTreeNode = {
      id: "root",
      label: "Root",
      kind: "modRoot",
      icon: "Mod",
      exists: true,
      children: [
        {
          id: "scene:arena",
          label: "Arena",
          kind: "scene",
          icon: "Play",
          exists: true,
          children: [],
        },
      ],
    };

    const merged = mergeProjectTrees({
      id: "root",
      label: "Root",
      kind: "modRoot",
      icon: "Mod",
      exists: true,
      children: [
        {
          id: "scene:arena",
          label: "Arena",
          kind: "scene",
          icon: "Play",
          exists: true,
          children: [
            {
              id: "scene-doc:arena",
              label: "scene.yml",
              kind: "sceneDocument",
              icon: "Yml",
              exists: true,
            },
          ],
        },
      ],
    }, fallback);

    expect(merged.children?.[0]).toMatchObject({
      id: "scene:arena",
      kind: "scene",
      children: [],
    });
  });

  it("matches search against nested child nodes", () => {
    const node: ProjectTreeNode = {
      id: "root",
      label: "Root",
      kind: "modRoot",
      icon: "Mod",
      exists: true,
      children: [
        {
          id: "scene:arena",
          label: "Arena",
          kind: "scene",
          icon: "Play",
          exists: true,
          path: "scenes/arena/scene.yml",
        },
      ],
    };

    expect(projectNodeMatchesSearch(node, "arena")).toBe(true);
    expect(projectNodeMatchesSearch(node, "missing")).toBe(false);
  });

  it("normalizes project-relative paths from absolute inputs", () => {
    expect(relativeProjectPath("D:/mods/test/scenes/arena/scene.yml")).toBe("scenes/arena/scene.yml");
    expect(relativeProjectPath("D:/mods/test/custom/config.yml")).toBe("custom/config.yml");
  });

  it("maps editor statuses and strips descriptor suffixes", () => {
    expect(statusForEditorStatus("missingDependency")).toBe("warn");
    expect(statusForEditorStatus("previewFailed")).toBe("error");
    expect(projectNodeKindLabel("sceneDocument")).toBe("scene document");
    expect(assetDisplayLabel(file("terrain.image.yml"))).toBe("terrain");
  });
});

function file(name: string): EditorProjectFileDto {
  return {
    name,
    path: `/mod/${name}`,
    relativePath: name,
    kind: "asset",
    isDir: false,
    sizeBytes: 0,
    children: [],
  };
}
