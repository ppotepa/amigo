import { describe, expect, it } from "vitest";
import { composeTraits } from "./traitComposition";
import type { EditorMetadataCatalogDto } from "./editorMetadataTypes";

describe("traitComposition", () => {
  it("resolves known trait descriptors", () => {
    const catalog = testCatalog();
    const traits = composeTraits(catalog, ["Renderable2D"]);
    expect(traits).toHaveLength(1);
    expect(traits[0].descriptor?.kind).toBe("Renderable2D");
  });

  it("returns null descriptor for unknown traits", () => {
    const catalog = testCatalog();
    const traits = composeTraits(catalog, ["UnknownTrait"]);
    expect(traits).toHaveLength(1);
    expect(traits[0].descriptor).toBeNull();
  });
});

function testCatalog(): EditorMetadataCatalogDto {
  return {
    components: [],
    metadataTraits: [
      {
        kind: "Renderable2D",
        label: "Renderable 2D",
        description: "Render content",
        appliesTo: ["Entity", "Component"],
        expectedYamlShapes: [],
        propertyGroups: [],
        controls: [],
        patchOps: [],
        diagnostics: [],
      },
    ],
  };
}
