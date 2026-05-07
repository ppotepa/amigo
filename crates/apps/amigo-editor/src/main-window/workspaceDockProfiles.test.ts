import { describe, expect, it } from "vitest";
import { editorComponentById } from "../editor-components/componentRegistry";
import {
  normalizeWorkspaceDockProfileId,
  workspaceDockProfileForComponent,
} from "./workspaceDockProfiles";

describe("workspaceDockProfiles", () => {
  it("resolves the UI document editor profile", () => {
    const profile = workspaceDockProfileForComponent(editorComponentById("ui.document.editor"));

    expect(profile.id).toBe("ui-document");
    expect(profile.left).toContain("ui.document.structure");
    expect(profile.rightTop).toContain("entity.properties");
    expect(profile.rightBottom).toContain("document.changes");
  });

  it("resolves the scene preview profile", () => {
    const profile = workspaceDockProfileForComponent(editorComponentById("scene.preview"));

    expect(profile.id).toBe("scene-editor");
    expect(profile.left).toContain("scene.hierarchy");
    expect(profile.rightTop).toContain("scene.context");
  });

  it("falls back to default editor profile for unknown ids", () => {
    expect(normalizeWorkspaceDockProfileId("missing")).toBe("default-editor");
  });
});
