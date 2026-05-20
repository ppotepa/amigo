import { remapClamp, smoothstep } from "../math";
import { type BodyPart, type ViewContext } from "../types";

export function computeFusionPolicy(part: BodyPart, ctx: ViewContext) {
  if (!part.fusionGroup) return { group: null, t: 0 };
  return {
    group: part.fusionGroup,
    t: smoothstep(remapClamp(ctx.side, 0.28, 0.68)),
  };
}
