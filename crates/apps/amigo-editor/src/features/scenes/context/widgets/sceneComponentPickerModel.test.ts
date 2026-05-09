import { describe, expect, it } from "vitest";
import type { EditorComponentDescriptorDto } from "../../../metadata/editorMetadataTypes";
import {
  canAttachDescriptorToScene,
  filterSceneComponentDescriptors,
} from "./sceneComponentPickerModel";

function descriptor(
  kind: string,
  ownerScopes?: EditorComponentDescriptorDto["ownerScopes"],
): EditorComponentDescriptorDto {
  return {
    kind,
    typeName: kind,
    label: kind,
    domains: ["Data"],
    ownerScopes,
    properties: [],
  };
}

describe("sceneComponentPickerModel", () => {
  it("filters descriptors by label and type", () => {
    const values = [descriptor("Sprite2D"), descriptor("InputActionMap")];
    expect(filterSceneComponentDescriptors(values, "input")).toEqual([values[1]]);
  });

  it("allows scene-scoped descriptors", () => {
    expect(canAttachDescriptorToScene(descriptor("InputActionMap", ["scene"]))).toEqual({ allowed: true });
  });

  it("rejects entity-only and missing owner scopes", () => {
    expect(canAttachDescriptorToScene(descriptor("Sprite2D", ["entity"])).allowed).toBe(false);
    expect(canAttachDescriptorToScene(descriptor("LegacyComponent")).allowed).toBe(false);
  });
});
