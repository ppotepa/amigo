import { describe, expect, it } from "vitest";
import type { EditorUiNodeDto } from "../../api/dto";
import {
  findUiNode,
  getSiblingInfo,
  validateAddNodeDraft,
} from "./uiDocumentEditorModel";

describe("uiDocumentEditorModel", () => {
  it("finds nested UI nodes by path", () => {
    expect(findUiNode(rootNode(), "root.panel.start")?.id).toBe("start");
  });

  it("validates add-node draft ids and required text", () => {
    expect(
      validateAddNodeDraft({
        parentPath: "root",
        kind: "button",
        id: "Bad ID",
        label: "Bad",
        text: "Click",
      }),
    ).toContain("lowercase");

    expect(
      validateAddNodeDraft({
        parentPath: "root",
        kind: "button",
        id: "valid-id",
        label: "Valid",
        text: "",
      }),
    ).toContain("Text is required");
  });

  it("returns sibling index and count", () => {
    expect(getSiblingInfo(rootNode(), "root.panel.options")).toEqual({
      parentPath: "root.panel",
      index: 1,
      count: 2,
    });
  });
});

function rootNode(): EditorUiNodeDto {
  return {
    path: "root",
    id: "root",
    kind: "column",
    label: "Root",
    style: {},
    enabled: true,
    visible: true,
    childCount: 1,
    children: [
      {
        path: "root.panel",
        id: "panel",
        kind: "panel",
        label: "Panel",
        style: {},
        enabled: true,
        visible: true,
        childCount: 2,
        children: [
          {
            path: "root.panel.start",
            id: "start",
            kind: "button",
            label: "Start",
            text: "START",
            style: {},
            enabled: true,
            visible: true,
            childCount: 0,
            children: [],
          },
          {
            path: "root.panel.options",
            id: "options",
            kind: "button",
            label: "Options",
            text: "OPTIONS",
            style: {},
            enabled: true,
            visible: true,
            childCount: 0,
            children: [],
          },
        ],
      },
    ],
  };
}
