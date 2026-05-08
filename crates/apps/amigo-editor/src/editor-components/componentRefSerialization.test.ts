import { describe, expect, it } from "vitest";
import { resolveSerializedComponentRef } from "./componentRefSerialization";
import { UiDocumentEditorComponent } from "./componentRegistry";

describe("component ref serialization", () => {
  it("rehydrates typed UI document context", () => {
    const resolved = resolveSerializedComponentRef({
      componentId: UiDocumentEditorComponent.id,
      context: {
        sceneId: "main-menu",
        entityId: "ui-root",
        componentIndex: "0",
        initialTemplate: "empty",
      },
    });

    expect(resolved?.component).toBe(UiDocumentEditorComponent);
    expect(resolved?.context?.componentIndex).toBe(0);
    expect(typeof resolved?.context?.componentIndex).toBe("number");
  });

  it("rejects unknown component ids", () => {
    expect(resolveSerializedComponentRef({ componentId: "missing.component" })).toBeNull();
  });
});
