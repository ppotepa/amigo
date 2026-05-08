import { describe, expect, it } from "vitest";
import {
  DocumentChangesComponent,
  EntityPropertiesComponent,
  SceneContextComponent,
  SceneHierarchyComponent,
  ScenePreviewComponent,
  UiDocumentEditorComponent,
  UiDocumentStructureComponent,
} from "../editor-components/componentRegistry";
import {
  normalizeWorkspaceDockProfileId,
  WORKSPACE_DOCK_PROFILES,
  workspaceDockProfileForComponent,
} from "./workspaceDockProfiles";

describe("workspaceDockProfiles", () => {
  it("resolves the UI document editor profile", () => {
    const profile = workspaceDockProfileForComponent(UiDocumentEditorComponent);

    expect(profile.id).toBe("ui-document");
    expect(profile.left).toContain(UiDocumentStructureComponent);
    expect(profile.rightTop).toContain(EntityPropertiesComponent);
    expect(profile.rightBottom).toContain(DocumentChangesComponent);
  });

  it("resolves the scene preview profile", () => {
    const profile = workspaceDockProfileForComponent(ScenePreviewComponent);

    expect(profile.id).toBe("scene-editor");
    expect(profile.left).toContain(SceneHierarchyComponent);
    expect(profile.rightTop).toContain(SceneContextComponent);
  });

  it("stores component definitions, not string ids", () => {
    for (const profile of Object.values(WORKSPACE_DOCK_PROFILES)) {
      for (const area of [profile.left, profile.rightTop, profile.rightBottom, profile.bottom]) {
        for (const component of area) {
          expect(typeof component).toBe("object");
          expect(typeof component.id).toBe("string");
          expect(typeof component.title).toBe("string");
        }
      }
    }
  });

  it("falls back to default editor profile for unknown ids", () => {
    expect(normalizeWorkspaceDockProfileId("missing")).toBe("default-editor");
  });
});
