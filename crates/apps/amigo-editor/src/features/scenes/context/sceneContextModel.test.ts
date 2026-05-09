import { describe, expect, it } from "vitest";
import { buildSceneContextModel } from "./sceneContextModel";
import type { BuildSceneContextModelInput } from "./sceneContextTypes";

const baseScene = {
  id: "scene-main",
  label: "Main",
  status: "valid",
  documentPath: "scenes/main.scene.yaml",
  scriptPath: "scripts/main.rhai",
  launcherVisible: true,
  diagnostics: [],
} as unknown as BuildSceneContextModelInput["scene"];

describe("sceneContextModel", () => {
  it("keeps asset usage in sceneInfo", () => {
    const model = buildSceneContextModel({
      scene: baseScene,
      managedAssets: [
        {
          kind: "spritesheet",
          domain: "spritesheet",
          assetId: "hero",
          assetKey: "hero",
          label: "Hero",
          references: [],
          usedBy: ["scenes/main.scene.yaml"],
        },
      ] as any,
      rawFiles: [],
    });

    expect(model.sceneInfo.assetCount).toBe(1);
    expect(model.sceneInfo.assetGroups[0]?.id).toBe("spritesheet");
  });

  it("groups scene entity components by domain", () => {
    const model = buildSceneContextModel({
      scene: baseScene,
      entities: [
        {
          id: "player",
          componentTypes: ["Sprite2DComponent", "RigidBody2DComponent", "Transform2DComponent"],
        },
      ] as any,
    });

    expect(model.components.total).toBe(3);
    expect(model.components.foldedHint).toContain("3 components");
    expect(model.components.groups.map((group) => group.id)).toEqual(
      expect.arrayContaining(["motion", "physics", "rendering"]),
    );
  });

  it("returns header badges and navigation folded hint", () => {
    const model = buildSceneContextModel({
      scene: baseScene,
      entities: [
        {
          id: "player-spawn",
          name: "Player Spawn",
          tags: [],
          groups: [],
          componentTypes: ["SpawnPointComponent"],
        },
        {
          id: "exit-trigger",
          name: "Exit Trigger",
          tags: [],
          groups: [],
          componentTypes: ["Trigger2DComponent"],
        },
      ] as any,
    });

    expect(model.header.badges.map((badge) => badge.id)).toContain("kind");
    expect(model.navigation.foldedHint).toContain("entries");
    expect(model.navigation.entries).toHaveLength(1);
    expect(model.navigation.triggers).toHaveLength(1);
  });

  it("returns entitiesInfo", () => {
    const model = buildSceneContextModel({
      scene: baseScene,
      entities: [
        {
          id: "camera",
          name: "Camera",
          tags: [],
          groups: [],
          visible: true,
          componentTypes: ["Camera2DComponent"],
        },
      ] as any,
    });

    expect(model.entitiesInfo.total).toBe(1);
    expect(model.entitiesInfo.visibleCount).toBe(1);
    expect(model.entitiesInfo.groups[0]?.id).toBe("camera");
  });

  it("summarizes diagnostics by level", () => {
    const model = buildSceneContextModel({
      scene: baseScene,
      diagnostics: [
        { level: "error", code: "E1", message: "Broken scene", path: "scenes/main.scene.yaml" },
        { level: "warning", code: "W1", message: "Risky scene", path: "scenes/main.scene.yaml" },
      ] as any,
    });

    expect(model.diagnosticsInfo.errorCount).toBe(1);
    expect(model.diagnosticsInfo.warningCount).toBe(1);
  });
});
