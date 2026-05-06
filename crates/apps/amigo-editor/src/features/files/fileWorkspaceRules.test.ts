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
