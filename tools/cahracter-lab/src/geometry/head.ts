import { applyPseudoDepth, boundsOf, centroid, lerp, lerpPoints, pointsToPath, remapClamp, smoothstep } from "../math";
import { buildEarRig, buildRig, computeEarPoints } from "../rig";
import { type AppState, type BodyPart, type EarKey, type GeometryPayload, type ViewState } from "../types";
import type { DetailGeometryResolver, MassGeometryResolver } from "./registry";

export const HEAD_MASS_GEOMETRY_STRATEGIES = ["head.faceMorph", "head.noseMorph", "head.earMorph"] as const;
export const HEAD_DETAIL_GEOMETRY_STRATEGIES = [
  "head.mouthLine",
  "head.noseHighlight",
  "head.nostril",
  "head.earInnerCurve",
] as const;

function getFaceRig(state: AppState, side: 1 | -1) {
  const key = `${side}:${state.samples}`;
  if (!state.rigs.has(key)) state.rigs.set(key, buildRig(side, state.samples));
  return state.rigs.get(key)!;
}

function getEarRig(state: AppState) {
  const key = `${state.samples}`;
  if (!state.earRigs.has(key)) state.earRigs.set(key, buildEarRig(state.samples));
  return state.earRigs.get(key)!;
}

export function evaluateHeadMassGeometry(
  part: BodyPart,
  state: AppState,
  viewState: ViewState,
  yaw: number,
  yawRad: number,
): GeometryPayload | null {
  if (!part.geometry) return null;
  const t = viewState.t;

  switch (part.geometry.strategy) {
    case "head.faceMorph": {
      const pair = getFaceRig(state, viewState.side).find(item => item.key === "face");
      if (!pair?.back) return null;
      const sideMorph = lerpPoints(pair.source, pair.target, t);
      const backBlend = smoothstep(remapClamp(viewState.back, 0.22, 1.0));
      const points = applyPseudoDepth(lerpPoints(sideMorph, pair.back, backBlend), yawRad, t, state.depth);
      return {
        points,
        path: pointsToPath(points, state.smooth),
        bounds: boundsOf(points),
        opacity: 1,
        visible: true,
        meta: {
          shadeOpacity: (0.75 + t * 0.12 + viewState.back * 0.08).toFixed(3),
          lightOpacity: (0.68 + t * 0.08 - viewState.back * 0.14).toFixed(3),
        },
      };
    }
    case "head.noseMorph": {
      const pair = getFaceRig(state, viewState.side).find(item => item.key === "nose");
      if (!pair) return null;
      const points = applyPseudoDepth(lerpPoints(pair.source, pair.target, t), yawRad, t, state.depth);
      return {
        points,
        path: pointsToPath(points, state.smooth),
        bounds: boundsOf(points),
        opacity: 1,
        visible: viewState.showNose,
      };
    }
    case "head.earMorph": {
      const earKey: EarKey = part.geometry.side === "left" ? "earLeft" : "earRight";
      const earState = viewState.ears.find(item => item.key === earKey);
      if (!earState) return null;
      const points = applyPseudoDepth(computeEarPoints(getEarRig(state)[earKey], yaw), yawRad, t, state.depth);
      return {
        points,
        path: pointsToPath(points, state.smooth),
        bounds: boundsOf(points),
        opacity: 1,
        visible: true,
        meta: {
          shadeOpacity: String((0.22 + 0.22 * (1 - earState.fusion) + 0.10 * (earState.isNear ? 1 : 0)) * (earState.isFar ? 0.72 : 1)),
          lightOpacity: String(0.28 + 0.12 * (1 - earState.fusion)),
        },
      };
    }
    default:
      return null;
  }
}

export function evaluateHeadDetailGeometry(
  part: BodyPart,
  parentGeometry: GeometryPayload | null,
  viewState: ViewState,
  outlineMode: string,
): GeometryPayload | null {
  if (!part.geometry) return null;

  switch (part.geometry.strategy) {
    case "head.mouthLine": {
      const facePoints = parentGeometry?.points;
      if (!facePoints) return null;
      const bounds = boundsOf(facePoints);
      const center = centroid(facePoints);
      const nearSign = viewState.side > 0 ? -1 : 1;
      const t = viewState.t;
      const mouthY = center.y + bounds.height * 0.23;
      const mouthW = lerp(bounds.width * 0.14, bounds.width * 0.09, t);
      const mouthTilt = viewState.side * t * bounds.height * 0.015;
      const mouthX = center.x + nearSign * t * bounds.width * 0.015;
      return {
        path: `M ${(mouthX - mouthW).toFixed(2)} ${(mouthY - mouthTilt).toFixed(2)} Q ${mouthX.toFixed(2)} ${(mouthY + bounds.height * 0.018).toFixed(2)}, ${(mouthX + mouthW).toFixed(2)} ${(mouthY + mouthTilt).toFixed(2)}`,
        bounds,
        visible: viewState.showMouth,
      };
    }
    case "head.noseHighlight": {
      const nosePoints = parentGeometry?.points;
      if (!nosePoints) return null;
      const bounds = boundsOf(nosePoints);
      const center = centroid(nosePoints);
      const sign = viewState.side > 0 ? -1 : 1;
      const t = viewState.t;
      return {
        path: `M ${(center.x - sign * bounds.width * 0.10).toFixed(2)} ${(bounds.minY + bounds.height * 0.20).toFixed(2)} Q ${(center.x - sign * bounds.width * 0.18).toFixed(2)} ${center.y.toFixed(2)}, ${(center.x - sign * bounds.width * 0.10).toFixed(2)} ${(bounds.minY + bounds.height * 0.84).toFixed(2)}`,
        bounds,
        opacity: 0.08 + t * 0.18,
        visible: viewState.showNose && outlineMode !== "SILHOUETTE_ONLY" && viewState.zone !== "BACK_PROXY" && viewState.profile > 0.5,
      };
    }
    case "head.nostril": {
      const nosePoints = parentGeometry?.points;
      if (!nosePoints) return null;
      const bounds = boundsOf(nosePoints);
      const center = centroid(nosePoints);
      const sign = viewState.side > 0 ? -1 : 1;
      const t = viewState.t;
      return {
        bounds,
        opacity: 0.04 + t * 0.18,
        visible: viewState.showNose && outlineMode !== "SILHOUETTE_ONLY" && viewState.zone !== "BACK_PROXY" && viewState.profile > 0.5,
        meta: {
          cx: (center.x + sign * bounds.width * 0.17).toFixed(2),
          cy: (bounds.minY + bounds.height * 0.72).toFixed(2),
          rx: Math.max(1.0, bounds.width * lerp(0.032, 0.055, t)).toFixed(2),
          ry: Math.max(0.6, bounds.height * lerp(0.015, 0.025, t)).toFixed(2),
        },
      };
    }
    case "head.earInnerCurve": {
      const earPoints = parentGeometry?.points;
      if (!earPoints) return null;
      const bounds = boundsOf(earPoints);
      const cx = bounds.minX + bounds.width * 0.52;
      const y1 = bounds.minY + bounds.height * 0.22;
      const y2 = bounds.minY + bounds.height * 0.78;
      const localSide = part.geometry.side === "right" ? 1 : -1;
      const earState = viewState.ears.find(item => item.key === (part.geometry?.side === "right" ? "earRight" : "earLeft"));
      if (!earState) return null;
      const curve = bounds.width * (localSide > 0 ? -0.18 : 0.18) * (earState.depth >= 0 ? 1 : -1);
      return {
        path: `M ${cx.toFixed(2)} ${y1.toFixed(2)} C ${(cx + curve).toFixed(2)} ${(y1 + bounds.height * 0.18).toFixed(2)}, ${(cx - curve * 0.65).toFixed(2)} ${(y2 - bounds.height * 0.18).toFixed(2)}, ${cx.toFixed(2)} ${y2.toFixed(2)}`,
        bounds,
        visible: outlineMode !== "SILHOUETTE_ONLY" && earState.isNear && viewState.profile < 0.68 && viewState.back < 0.58,
      };
    }
    default:
      return null;
  }
}

export const headMassGeometryResolverEntries: Record<string, MassGeometryResolver> = Object.fromEntries(
  HEAD_MASS_GEOMETRY_STRATEGIES.map(strategy => [strategy, evaluateHeadMassGeometry]),
) as Record<string, MassGeometryResolver>;

export const headDetailGeometryResolverEntries: Record<string, DetailGeometryResolver> = Object.fromEntries(
  HEAD_DETAIL_GEOMETRY_STRATEGIES.map(strategy => [strategy, evaluateHeadDetailGeometry]),
) as Record<string, DetailGeometryResolver>;
