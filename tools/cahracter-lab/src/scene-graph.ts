import { remapClamp, smoothstep } from "./math";
import { computeFusionPolicy } from "./policies/fusion";
import { computeRenderPasses as computeSortedRenderPasses, computeRenderPassPolicy } from "./policies/layer";
import { computeVisibilityPolicy } from "./policies/visibility";
import { PIVOT_X, PIVOT_Y, type BodyPart, type EvaluatedBodyPart, type ViewContext } from "./types";

export function computeViewContext(angle: number): ViewContext {
  const theta = (angle * Math.PI) / 180;
  const c = Math.cos(theta);
  const s = Math.sin(theta);
  return {
    angle,
    theta,
    c,
    s,
    front: Math.max(0, c),
    back: Math.max(0, -c),
    side: Math.abs(s),
    dir: s >= 0 ? 1 : -1,
    profileT: smoothstep(Math.abs(s)),
    backT: smoothstep(Math.max(0, -c)),
  };
}

export function projectPoint2_5D(anchor: BodyPart["anchor"], ctx: ViewContext) {
  return {
    x: PIVOT_X + (anchor.x * ctx.c + anchor.z * ctx.s),
    y: PIVOT_Y + anchor.y,
    depth: anchor.z * ctx.c - anchor.x * ctx.s,
  };
}

export function evaluateBodyPart(part: BodyPart, ctx: ViewContext): EvaluatedBodyPart {
  const projected = projectPoint2_5D(part.anchor, ctx);
  const children = (part.children ?? []).map(child => evaluateBodyPart(child, ctx));
  return {
    id: part.id,
    type: part.type,
    source: part,
    projected,
    visible: computeVisibilityPolicy(part, ctx),
    depth: projected.depth,
    renderPass: computeRenderPassPolicy(part, projected.depth),
    fusion: computeFusionPolicy(part, ctx),
    geometry: null,
    children,
  };
}

export function flattenParts(root: EvaluatedBodyPart): EvaluatedBodyPart[] {
  const out: EvaluatedBodyPart[] = [root];
  for (const child of root.children) out.push(...flattenParts(child));
  return out;
}

export function computeFusionGroups(parts: EvaluatedBodyPart[]): Map<string, EvaluatedBodyPart[]> {
  const groups = new Map<string, EvaluatedBodyPart[]>();
  for (const part of parts) {
    if (!part.fusion.group) continue;
    const list = groups.get(part.fusion.group) ?? [];
    list.push(part);
    groups.set(part.fusion.group, list);
  }
  return groups;
}

export function computeRenderPasses(parts: EvaluatedBodyPart[]): EvaluatedBodyPart[] {
  return computeSortedRenderPasses(parts);
}
