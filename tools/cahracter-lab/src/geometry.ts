import { detailGeometryResolvers, massGeometryResolvers } from "./geometry/registry";
import { type AppState, type BodyPart, type GeometryPayload, type ViewState } from "./types";

export function evaluateMassGeometry(
  part: BodyPart,
  state: AppState,
  viewState: ViewState,
  yaw: number,
  yawRad: number,
): GeometryPayload | null {
  const strategy = part.geometry?.strategy;
  if (!strategy) return null;
  const resolver = massGeometryResolvers[strategy];
  return resolver ? resolver(part, state, viewState, yaw, yawRad) : null;
}

export function evaluateDetailGeometry(
  part: BodyPart,
  parentGeometry: GeometryPayload | null,
  viewState: ViewState,
  outlineMode: string,
): GeometryPayload | null {
  const strategy = part.geometry?.strategy;
  if (!strategy) return null;
  const resolver = detailGeometryResolvers[strategy];
  return resolver ? resolver(part, parentGeometry, viewState, outlineMode) : null;
}
