import { type BodyPart, type EvaluatedBodyPart, type RenderPass } from "../types";

export function computeRenderPassPolicy(part: BodyPart, depth: number): RenderPass {
  if (part.renderPass && part.type !== "mass") return part.renderPass;
  if (part.type === "detail") return "detail";
  if (part.type === "contour") return "outline";
  if (part.type === "mass") {
    if (depth < -18) return "farMass";
    if (depth > 18) return "nearMass";
    if (part.renderPass === "farMass" || part.renderPass === "nearMass") return part.renderPass;
    return "midMass";
  }
  if (part.renderPass) return part.renderPass;
  if (part.type === "group") return "midMass";
  if (depth < -12) return "farMass";
  if (depth > 12) return "nearMass";
  return "midMass";
}

export function computeRenderPasses(parts: EvaluatedBodyPart[]): EvaluatedBodyPart[] {
  const order: Record<RenderPass, number> = {
    farMass: 0,
    midMass: 1,
    nearMass: 2,
    detail: 3,
    outline: 4,
  };
  return [...parts]
    .filter(part => part.visible && part.type !== "group")
    .sort((a, b) => {
      const passDiff = order[a.renderPass] - order[b.renderPass];
      if (passDiff !== 0) return passDiff;
      const zDiff = (a.source.render?.zIndex ?? 0) - (b.source.render?.zIndex ?? 0);
      if (zDiff !== 0) return zDiff;
      return a.depth - b.depth;
    });
}
