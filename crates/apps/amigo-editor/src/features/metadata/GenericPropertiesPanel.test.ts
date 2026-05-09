import { describe, expect, it } from "vitest";
import {
  findComponentDescriptor,
  formatPropertyValue,
} from "./GenericPropertiesPanel";
import type { EditorMetadataCatalogDto } from "./editorMetadataTypes";

describe("GenericPropertiesPanel helpers", () => {
  it("formats readonly values consistently", () => {
    expect(formatPropertyValue(null)).toBe("-");
    expect(formatPropertyValue("hero")).toBe("hero");
    expect(formatPropertyValue(42)).toBe("42");
    expect(formatPropertyValue({ x: 1 })).toBe('{"x":1}');
  });

  it("finds descriptors by kind or type name", () => {
    const catalog = {
      components: [
        {
          kind: "Sprite2D",
          typeName: "Sprite2DComponent",
          label: "Sprite",
          domains: ["Render2D"],
          properties: [],
        },
      ],
    } as EditorMetadataCatalogDto;

    expect(findComponentDescriptor(catalog, "Sprite2D")?.label).toBe("Sprite");
    expect(findComponentDescriptor(catalog, "Sprite2DComponent")?.label).toBe("Sprite");
  });
});
