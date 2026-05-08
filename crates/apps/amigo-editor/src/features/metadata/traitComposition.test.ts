import { describe, expect, it } from "vitest";
import { composeEditorSectionsForTraits } from "./traitComposition";
import type { EditorMetadataCatalogDto } from "./editorMetadataTypes";

describe("traitComposition", () => {
  it("composes editor sections from metadata traits", () => {
    const catalog = {
      components: [],
      metadataTraits: [
        {
          kind: "Renderable2D",
          label: "Renderable 2D",
          description: "Render content",
          appliesTo: ["Entity", "Component"],
          expectedYamlShapes: [],
          propertyGroups: [],
          editorSections: [
            {
              id: "render2d.summary",
              label: "Render2D",
              placement: "RightTop",
              description: "Render summary",
              priority: 200,
              defaultExpanded: true,
            },
          ],
          controls: [],
          patchOps: [],
          diagnostics: [],
        },
      ],
    } satisfies EditorMetadataCatalogDto;

    const sections = composeEditorSectionsForTraits(catalog, ["Renderable2D"]);

    expect(sections).toHaveLength(1);
    expect(sections[0].id).toBe("render2d.summary");
    expect(sections[0].traitKind).toBe("Renderable2D");
  });
});
