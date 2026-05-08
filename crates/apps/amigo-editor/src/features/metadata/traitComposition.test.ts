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
            {
              id: "render2d.properties",
              label: "Render Properties",
              placement: "RightBottom",
              description: "Render properties",
              priority: 100,
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

    expect(sections).toHaveLength(2);
    expect(sections.some((section) => section.id === "render2d.summary")).toBe(true);
    expect(sections.every((section) => section.traitKind === "Renderable2D")).toBe(true);
  });

  it("filters sections by placement", () => {
    const catalog = testCatalog();

    const sections = composeEditorSectionsForTraits(catalog, ["Renderable2D"], "RightTop");

    expect(sections).toHaveLength(1);
    expect(sections[0].placement).toBe("RightTop");
  });

  it("sorts sections by priority", () => {
    const catalog = testCatalog();

    const sections = composeEditorSectionsForTraits(catalog, ["Renderable2D"]);

    expect(sections.map((section) => section.id)).toEqual([
      "render2d.properties",
      "render2d.summary",
    ]);
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
        editorSections: [
          {
            id: "render2d.summary",
            label: "Render2D",
            placement: "RightTop",
            description: "Render summary",
            priority: 200,
            defaultExpanded: true,
          },
          {
            id: "render2d.properties",
            label: "Render Properties",
            placement: "RightBottom",
            description: "Render properties",
            priority: 100,
            defaultExpanded: true,
          },
        ],
        controls: [],
        patchOps: [],
        diagnostics: [],
      },
    ],
  };
}
