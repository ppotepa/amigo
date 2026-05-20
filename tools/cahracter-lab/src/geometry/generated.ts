import { applyPseudoDepth, boundsOf, pointsToPath } from "../math";
import { type AppState, type BodyPart, type GeometryPayload, type Point, type ViewState } from "../types";
import type { MassGeometryResolver } from "./registry";

export const GENERATED_MASS_GEOMETRY_STRATEGIES = ["generated.softMass"] as const;

function numberParam(part: BodyPart, key: string, fallback: number): number {
  const value = part.geometry?.params?.[key];
  return typeof value === "number" ? value : fallback;
}

function buildSoftMassContour(part: BodyPart, viewState: ViewState): Point[] {
  const halfTop = numberParam(part, "topWidth", 48) * 0.5;
  const halfBottom = numberParam(part, "bottomWidth", 72) * 0.5;
  const height = numberParam(part, "height", 140);
  const shoulderDrop = numberParam(part, "shoulderDrop", height * 0.18);
  const waistLift = numberParam(part, "waistLift", height * 0.10);
  const profilePinch = numberParam(part, "profilePinch", 0.28);
  const sideShift = numberParam(part, "sideShift", 10) * viewState.side;

  const pinch = 1 - viewState.t * profilePinch;
  const top = halfTop * pinch;
  const bottom = halfBottom * pinch;
  const cx = part.anchor.x + sideShift;
  const topY = part.anchor.y - height * 0.5;
  const bottomY = part.anchor.y + height * 0.5;

  return [
    { x: cx - top, y: topY + shoulderDrop * 0.15 },
    { x: cx - top * 1.08, y: topY + shoulderDrop * 0.62 },
    { x: cx - bottom * 0.94, y: bottomY - waistLift * 0.55 },
    { x: cx - bottom, y: bottomY - waistLift * 0.08 },
    { x: cx - bottom * 0.34, y: bottomY },
    { x: cx + bottom * 0.34, y: bottomY },
    { x: cx + bottom, y: bottomY - waistLift * 0.08 },
    { x: cx + bottom * 0.94, y: bottomY - waistLift * 0.55 },
    { x: cx + top * 1.08, y: topY + shoulderDrop * 0.62 },
    { x: cx + top, y: topY + shoulderDrop * 0.15 },
    { x: cx + top * 0.28, y: topY },
    { x: cx - top * 0.28, y: topY },
  ];
}

function evaluateGeneratedMassGeometry(
  part: BodyPart,
  state: AppState,
  viewState: ViewState,
  _yaw: number,
  yawRad: number,
): GeometryPayload | null {
  if (!part.geometry) return null;

  switch (part.geometry.strategy) {
    case "generated.softMass": {
      const points = applyPseudoDepth(buildSoftMassContour(part, viewState), yawRad, viewState.t, state.depth);
      return {
        points,
        path: pointsToPath(points, state.smooth),
        bounds: boundsOf(points),
        opacity: 1,
        visible: true,
        meta: {
          shadeOpacity: (0.64 + viewState.t * 0.14 + viewState.back * 0.04).toFixed(3),
          lightOpacity: (0.56 + (1 - viewState.back) * 0.10).toFixed(3),
        },
      };
    }
    default:
      return null;
  }
}

export const generatedMassGeometryResolverEntries: Record<string, MassGeometryResolver> = Object.fromEntries(
  GENERATED_MASS_GEOMETRY_STRATEGIES.map(strategy => [strategy, evaluateGeneratedMassGeometry]),
) as Record<string, MassGeometryResolver>;
