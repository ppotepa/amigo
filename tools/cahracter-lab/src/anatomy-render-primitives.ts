import { generateBodyPartPrimitives, generatePolicyOutlinePrimitives } from "./render-primitive-generator";
import { type OutlinePolicy, type EvaluatedBodyPart } from "./types";
import { type RenderPrimitive } from "./render-primitives";

export function generateAnatomyRenderPrimitives(
  parts: EvaluatedBodyPart[],
  policy: OutlinePolicy,
): RenderPrimitive[] {
  return [
    ...generateBodyPartPrimitives(parts),
    ...generatePolicyOutlinePrimitives(parts, policy),
  ];
}

export function describePrimitiveOrder(primitives: RenderPrimitive[]): string {
  const labels = primitives
    .filter(primitive => primitive.visible)
    .map(primitive => primitive.sourcePartId)
    .filter((label, index, items) => items.indexOf(label) === index);
  return `order: ${labels.join(" -> ")}`;
}
