import { describe, expect, it } from "vitest";
import type { EditorProjectFileDto } from "../../api/dto";
import { editorComponentById } from "../../editor-components/componentRegistry";
import { resolveFileWorkspaceDescriptor } from "./fileWorkspaceRules";

describe("fileWorkspaceRules", () => {
  it("resolves tile rulesets to a registered workspace component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("spritesheets/dirt/rulesets/platform/solid.yml", "tileset"));

    expect(descriptor.componentId).toBe("file.tile-ruleset");
    expect(editorComponentById(descriptor.componentId)?.id).toBe("file.tile-ruleset");
  });

  it("resolves image asset descriptors to a registered editor component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("images/logo/image.yml", "imageAsset"));

    expect(descriptor.componentId).toBe("file.image-asset");
    expect(editorComponentById(descriptor.componentId)?.id).toBe("file.image-asset");
  });

  it("resolves spritesheets to a registered sheet editor component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("spritesheets/player/spritesheet.yml", "spritesheet"));

    expect(descriptor.componentId).toBe("file.sprite");
    expect(editorComponentById(descriptor.componentId)?.id).toBe("file.sprite");
  });

  it("resolves tilemaps to a registered tilemap editor component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("tilemaps/level-01/tilemap.yml", "tilemap"));

    expect(descriptor.componentId).toBe("file.tilemap");
    expect(editorComponentById(descriptor.componentId)?.id).toBe("file.tilemap");
  });

  it("resolves scene yaml to a registered file component", () => {
    const descriptor = resolveFileWorkspaceDescriptor(file("scenes/main-menu/scene.yml", "scene"));

    expect(descriptor.componentId).toBe("file.scene");
    expect(editorComponentById(descriptor.componentId)?.id).toBe("file.scene");
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
