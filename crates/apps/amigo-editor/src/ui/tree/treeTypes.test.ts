import { describe, expect, it } from "vitest";
import { TREE_NODE_CAPABILITIES_NONE } from "./treeTypes";

describe("TREE_NODE_CAPABILITIES_NONE", () => {
  it("disables every optional tree behavior", () => {
    expect(Object.values(TREE_NODE_CAPABILITIES_NONE).every((value) => value === false)).toBe(true);
  });
});
