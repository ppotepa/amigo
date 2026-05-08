import { describe, expectTypeOf, it } from "vitest";
import {
  FileTextureComponent,
  UiDocumentEditorComponent,
} from "./componentRegistry";
import type {
  EditorComponentContextOf,
  FileWorkspaceComponentContext,
  UiDocumentEditorContext,
} from "./componentTypes";

describe("typed editor component definitions", () => {
  it("keeps context type on component consts", () => {
    expectTypeOf<EditorComponentContextOf<typeof UiDocumentEditorComponent>>()
      .toEqualTypeOf<UiDocumentEditorContext>();

    expectTypeOf<EditorComponentContextOf<typeof FileTextureComponent>>()
      .toEqualTypeOf<FileWorkspaceComponentContext>();
  });
});
