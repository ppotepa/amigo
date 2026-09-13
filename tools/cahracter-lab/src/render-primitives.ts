import { type Point, type RenderPass } from "./types";

export type PrimitiveLayer = "rig" | "outline" | "debug";
export type PrimitiveKind = "path" | "ellipse" | "circle";

export type PrimitiveRole =
  | "fill"
  | "shade"
  | "light"
  | "wire"
  | "detail"
  | "contour"
  | "silhouette"
  | "debug";

export type RenderPrimitive = {
  id: string;
  sourcePartId: string;
  kind: PrimitiveKind;
  role: PrimitiveRole;
  layer: PrimitiveLayer;
  pass: RenderPass;
  zIndex: number;
  depth: number;
  visible: boolean;
  className?: string;
  path?: string;
  point?: Point;
  attrs?: Record<string, string | number | boolean>;
  opacity?: number | string;
};

export type PrimitiveOutputSpec = {
  id?: string;
  kind: PrimitiveKind;
  role: PrimitiveRole;
  layer?: PrimitiveLayer;
  pass?: RenderPass;
  className?: string;
  zIndexOffset?: number;
  opacity?: number;
  opacityMeta?: string;
  attrs?: Record<string, string | number | boolean>;
  metaAttrs?: string[];
};

export type OutlineRenderHints = {
  contourClassName?: string;
  contourZIndex?: number;
  silhouetteClassName?: string;
  silhouetteZIndex?: number;
};

export type RenderHints = {
  material?: "skin" | "line" | "shadow" | "light" | "debug";
  zIndex?: number;
  primaryLighting?: boolean;
  debugSamplePoints?: boolean;
  outputs?: PrimitiveOutputSpec[];
  outline?: OutlineRenderHints;
};

const PASS_ORDER: Record<RenderPass, number> = {
  farMass: 0,
  midMass: 1,
  nearMass: 2,
  detail: 3,
  outline: 4,
};

export function sortRenderPrimitives(items: RenderPrimitive[]): RenderPrimitive[] {
  return [...items].sort((a, b) => {
    const passDiff = PASS_ORDER[a.pass] - PASS_ORDER[b.pass];
    if (passDiff !== 0) return passDiff;

    const zDiff = a.zIndex - b.zIndex;
    if (zDiff !== 0) return zDiff;

    return a.depth - b.depth;
  });
}
