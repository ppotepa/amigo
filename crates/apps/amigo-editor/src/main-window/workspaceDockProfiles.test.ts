import { describe, expect, it } from "vitest";
import {
  DocumentChangesComponent,
  ScenePreviewComponent,
  TargetPanelComponent,
  UiDocumentEditorComponent,
} from "../editor-components/componentRegistry";
import {
  componentSlot,
  normalizeWorkspaceDockProfileId,
  WORKSPACE_DOCK_PROFILES,
  workspaceDockProfileForComponent,
} from "./workspaceDockProfiles";

describe("workspaceDockProfiles", () => {
  it("resolves the UI document editor profile", () => {
    const profile = workspaceDockProfileForComponent(UiDocumentEditorComponent);

    expect(profile.id).toBe("ui-document");
    expect(profile.rightTop).toEqual([componentSlot(TargetPanelComponent)]);
    expect(profile.rightBottom).toContainEqual(componentSlot(DocumentChangesComponent));
  });

  it("resolves the scene preview profile", () => {
    const profile = workspaceDockProfileForComponent(ScenePreviewComponent);

    expect(profile.id).toBe("scene-editor");
    expect(profile.rightTop).toEqual([componentSlot(TargetPanelComponent)]);
  });

  it("stores dock slots", () => {
    for (const profile of Object.values(WORKSPACE_DOCK_PROFILES)) {
      for (const area of [profile.left, profile.rightTop, profile.rightBottom, profile.bottom]) {
        for (const slot of area) {
          expect(typeof slot).toBe("object");
          expect(typeof slot.kind).toBe("string");
          if (slot.kind === "component") {
            expect(typeof slot.component.id).toBe("string");
            expect(typeof slot.component.title).toBe("string");
          }
        }
      }
    }
  });

  it("falls back to default editor profile for unknown ids", () => {
    expect(normalizeWorkspaceDockProfileId("missing")).toBe("default-editor");
  });
});
