import { type BodyPart, type ViewContext } from "../types";

export function computeVisibilityPolicy(part: BodyPart, ctx: ViewContext): boolean {
  switch (part.visibilityMode ?? "always") {
    case "front-only":
      return ctx.back < 0.06 && ctx.side < 0.22;
    case "front-profile":
      return ctx.back < 0.38;
    case "front-side":
      return ctx.back < 0.5;
    case "not-back":
      return ctx.back < 0.62;
    default:
      return true;
  }
}
