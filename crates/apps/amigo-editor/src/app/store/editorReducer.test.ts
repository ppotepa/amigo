import { describe, expect, it } from "vitest";
import type { EditorModDetailsDto } from "../../api/dto";
import { reducer } from "./editorReducer";
import { initialState } from "./editorState";

describe("editorReducer", () => {
  it("ignores mod details loaded for a stale selected mod", () => {
    const state = reducer(initialState, { type: "modSelected", modId: "they-are-rotten" });

    const next = reducer(state, {
      type: "modDetailsLoaded",
      details: modDetails("playground-2d", "basic-scripting-demo"),
    });

    expect(next.modDetails).toBeNull();
    expect(next.selection).toEqual({ kind: "mod", modId: "they-are-rotten" });
  });

  it("accepts mod details for the selected mod", () => {
    const state = reducer(initialState, { type: "modSelected", modId: "they-are-rotten" });

    const next = reducer(state, {
      type: "modDetailsLoaded",
      details: modDetails("they-are-rotten", "main-menu"),
    });

    expect(next.modDetails?.id).toBe("they-are-rotten");
    expect(next.selection).toEqual({
      kind: "scene",
      modId: "they-are-rotten",
      sceneId: "main-menu",
    });
  });

  it("stores center component tabs and activates the opened tab", () => {
    const next = reducer(initialState, {
      type: "centerComponentTabOpened",
      instance: {
        instanceId: "ui.document.editor:test",
        componentId: "ui.document.editor",
        placement: { kind: "centerTab" },
        context: { sceneId: "main-menu" },
      },
    });

    expect(next.workspaces.main?.centerComponentTabs).toHaveLength(1);
    expect(next.workspaces.main?.activeTabId).toBe("ui.document.editor:test");
  });

  it("closes center component tabs and returns focus to scene preview", () => {
    const opened = reducer(initialState, {
      type: "centerComponentTabOpened",
      instance: {
        instanceId: "ui.document.editor:test",
        componentId: "ui.document.editor",
        placement: { kind: "centerTab" },
      },
    });

    const next = reducer(opened, {
      type: "centerComponentTabClosed",
      instanceId: "ui.document.editor:test",
    });

    expect(next.workspaces.main?.centerComponentTabs).toHaveLength(0);
    expect(next.workspaces.main?.activeTabId).toBe("scene-preview");
  });
});

function modDetails(modId: string, sceneId: string): EditorModDetailsDto {
  return {
    id: modId,
    name: modId,
    version: "0.1.0",
    description: null,
    authors: [],
    rootPath: `/mods/${modId}`,
    projectCacheId: modId,
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
      fonts: 0,
      scripts: 0,
      packages: 0,
      unknownFiles: 0,
    },
    scenes: [
      {
        id: sceneId,
        label: sceneId,
        description: null,
        path: `scenes/${sceneId}`,
        documentPath: `scenes/${sceneId}/scene.yml`,
        scriptPath: "",
        launcherVisible: true,
        status: "valid",
        previewCacheKey: sceneId,
        previewFps: 60,
        diagnostics: [],
        previewImageUrl: null,
      },
    ],
  };
}
