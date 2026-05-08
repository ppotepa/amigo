import { describe, expect, it } from "vitest";
import type { EditorProjectFileDto } from "../../api/dto";
import { resolveFileWorkspaceDescriptor } from "./fileWorkspaceRules";

describe("fileWorkspaceRules", () => {
  it("resolves tile rulesets to a registered workspace component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("spritesheets/dirt/rulesets/platform/solid.yml", "tileset"));

    expect(descriptor.component.id).toBe("file.tile-ruleset");
  });

  it("resolves image asset descriptors to a registered editor component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("images/logo/image.yml", "imageAsset"));

    expect(descriptor.component.id).toBe("file.image-asset");
  });

  it("resolves spritesheets to a registered sheet editor component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("spritesheets/player/spritesheet.yml", "spritesheet"));

    expect(descriptor.component.id).toBe("file.sprite");
  });

  it("resolves tilemaps to a registered tilemap editor component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("tilemaps/level-01/tilemap.yml", "tilemap"));

    expect(descriptor.component.id).toBe("file.tilemap");
  });

  it("resolves scene yaml to a registered file component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("scenes/main-menu/scene.yml", "scene"));

    expect(descriptor.component.id).toBe("file.scene");
  });

  it("resolves UI menu yaml to the UI document editor", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("ui/menus/main-menu.yml", "ui"));

    expect(descriptor.component.id).toBe("ui.document.editor");
  });

  it("resolves UI document yaml to the UI document editor", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("ui/documents/hud.yml", "ui"));

    expect(descriptor.component.id).toBe("ui.document.editor");
  });
});

function file(relativePath: string, kind = "yaml"): EditorProjectFileDto {
  return {
    name: relativePath.split("/").pop() ?? relativePath,
    path: `/mod/${relativePath}`,
    relativePath,
    kind,
    isDir: false,
    sizeBytes: 0,
    children: [],
  };
}
