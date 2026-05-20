import { type BodyPart, type ViewContext } from "../types";

export function computeVisibilityPolicy(part: BodyPart, ctx: ViewContext): boolean {
  switch (part.visibilityMode ?? "always") {
    case "front-only":
      return ctx.back < 0.1 && ctx.side < 0.3;
    case "front-profile":
      return ctx.back < 0.45;
    case "front-side":
      return ctx.back < 0.62;
    case "not-back":
      return ctx.back < 0.82;
    default:
      return true;
  }
}
