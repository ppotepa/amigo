import { describe, expect, it } from "vitest";
import type { EditorModDetailsDto } from "../../api/dto";
import { initialState } from "./editorState";
import { selectedScene } from "./editorSelectors";

describe("editorSelectors", () => {
  it("does not fall back to the first scene for asset selections", () => {
    expect(selectedScene({
      ...initialState,
      modDetails: modDetails(),
      selection: {
        kind: "asset",
        modId: "ink-wars",
        assetKey: "ink-wars/fonts/debug-placeholder",
        filePath: "fonts/debug-placeholder/font.yml",
      },
    })).toBeNull();
  });
});

function modDetails(): EditorModDetailsDto {
  return {
    id: "ink-wars",
    name: "Ink Wars",
    version: "0.1.0",
    description: null,
    authors: [],
    rootPath: "/mods/ink-wars",
    projectCacheId: "ink-wars",
    status: "valid",
    diagnostics: [],
    dependencies: [],
    missingDependencies: [],
    capabilities: [],
    sceneCount: 1,
    visibleSceneCount: 1,
    previewStatus: "ready",
    contentSummary: {
      totalFiles: 1,
      scenes: 1,
      sceneYaml: 1,
      spritesheets: 0,
      tilesets: 0,
      tilemaps: 0,
      textures: 0,
      audio: 0,
      fonts: 1,
      scripts: 0,
      packages: 0,
      unknownFiles: 0,
    },
    scenes: [{
      id: "pen-blue-v1",
      label: "Pen Blue V1",
      description: null,
      path: "scenes/pen-blue-v1",
      documentPath: "scenes/pen-blue-v1/scene.yml",
      scriptPath: "",
      launcherVisible: true,
      status: "valid",
      previewCacheKey: "pen-blue-v1",
      previewFps: 60,
      diagnostics: [],
      previewImageUrl: null,
    }],
  };
}
