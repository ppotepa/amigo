import { describe, expect, it } from "vitest";
import {
  editorComponentById,
  UiDocumentEditorComponent,
} from "./componentRegistry";

describe("editor component registry", () => {
  it("rehydrates serialized component ids", () => {
    expect(editorComponentById(UiDocumentEditorComponent.id)).toBe(UiDocumentEditorComponent);
  });

  it("rejects unknown component ids", () => {
    expect(editorComponentById("missing.component")).toBeUndefined();
  });
});
