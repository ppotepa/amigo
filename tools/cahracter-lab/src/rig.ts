import { PIVOT_X, type Contour, type EarKey, type EarRig, type EarState, type MorphPair, type Point, type ViewState } from "./types";
import {
  boundsOf,
  centroid,
  clamp,
  distanceSq,
  interpolatePointKeyframes,
  lerpPoints,
  mirrorPoints,
  normalized,
  remapClamp,
  reversePoints,
  rotatePoints,
  signedArea,
  smoothstep,
} from "./math";

const SOURCE = {
  centerFace: "#CENTER #FACE",
  centerRightEar: "#CENTER #PRAWE",
  centerLeftEar: "#CENTER #LEWE",
  centerNose: "#CENTER #NOSE",
  angleFace: "#ANGLE #ANGLE_FACE",
  angleNose: "#ANGLE #ANGLE_NOSE",
  angleEar: "#ANGLE #ANGLE_EAR",
  profileFace: "#LEFT_PROFILE #FACE_PROFILE",
  profileNose: "#LEFT_PROFILE #NOSE-PROFILE",
  profileEar: "#LEFT_PROFILE #EAR_LEFT",
  backFace: "#BACK #BACK_FACE",
  backLeftEar: "#BACK #BACK_LEFT_EAR",
  backRightEar: "#BACK #BACK_RIGHT_EAR",
} as const;

const EAR_DEFS: Record<EarKey, { localSide: 1 | -1; frontSelector: string; backSelector: string; label: string }> = {
  earRight: { localSide: 1, frontSelector: SOURCE.centerRightEar, backSelector: SOURCE.backLeftEar, label: "right" },
  earLeft: { localSide: -1, frontSelector: SOURCE.centerLeftEar, backSelector: SOURCE.backRightEar, label: "left" },
};

function samplePath(path: SVGPathElement, count: number, mirrored = false): Contour {
  const total = path.getTotalLength();
  const matrix = path.getCTM();
  let points: Point[] = [];
  for (let i = 0; i < count; i++) {
    const point = path.getPointAtLength((i / count) * total);
    if (matrix) {
      const worldPoint = new DOMPoint(point.x, point.y).matrixTransform(matrix);
      points.push({ x: worldPoint.x, y: worldPoint.y });
    } else {
      points.push({ x: point.x, y: point.y });
    }
  }
  if (mirrored) points = mirrorPoints(points);
  return { id: path.id || "path", length: total, points, center: centroid(points), area: signedArea(points) };
}

function sampleSelector(selector: string, count: number, mirrored = false): Contour {
  const element = document.querySelector<SVGPathElement>(selector);
  if (!element) throw new Error(`Brak path: ${selector}`);
  return samplePath(element, count, mirrored);
}

function alignTargetToSource(sourcePoints: Point[], targetPoints: Point[]): Point[] {
  const length = sourcePoints.length;
  const sourceNorm = normalized(sourcePoints);
  const candidates = [targetPoints, reversePoints(targetPoints)];
  let best: { score: number; reversed: boolean; offset: number } | null = null;

  for (const candidate of candidates) {
    const targetNorm = normalized(candidate);
    for (let offset = 0; offset < length; offset++) {
      let score = 0;
      for (let i = 0; i < length; i += 2) score += distanceSq(sourceNorm[i], targetNorm[(i + offset) % length]);
      if (!best || score < best.score) best = { score, reversed: candidate !== targetPoints, offset };
    }
  }

  const oriented = best?.reversed ? reversePoints(targetPoints) : targetPoints;
  return rotatePoints(oriented, best?.offset ?? 0);
}

function extremeIndex(points: Point[], axis: "top" | "bottom" | "left" | "right"): number {
  let bestIndex = 0;
  for (let i = 1; i < points.length; i++) {
    const point = points[i];
    const best = points[bestIndex];
    const better =
      axis === "top" ? point.y < best.y :
      axis === "bottom" ? point.y > best.y :
      axis === "left" ? point.x < best.x :
      point.x > best.x;
    if (better) bestIndex = i;
  }
  return bestIndex;
}

function alignByAnchor(sourcePoints: Point[], targetPoints: Point[], anchor: "top" | "bottom" | "left" | "right"): Point[] {
  const base = alignTargetToSource(sourcePoints, targetPoints);
  const sourceIndex = extremeIndex(sourcePoints, anchor);
  const targetIndex = extremeIndex(base, anchor);
  return rotatePoints(base, ((sourceIndex - targetIndex) % base.length + base.length) % base.length);
}

function fitContourToReference(
  reference: Point[],
  candidate: Point[],
  fitX: number,
  fitY: number,
  anchor: "center" | "bottom" = "center",
): Point[] {
  const referenceBounds = boundsOf(reference);
  const candidateBounds = boundsOf(candidate);
  const referenceCenter = centroid(reference);
  const candidateCenter = centroid(candidate);

  const targetScaleX = candidateBounds.width > 0 ? referenceBounds.width / candidateBounds.width : 1;
  const targetScaleY = candidateBounds.height > 0 ? referenceBounds.height / candidateBounds.height : 1;
  const scaleX = 1 + (targetScaleX - 1) * fitX;
  const scaleY = 1 + (targetScaleY - 1) * fitY;

  const scaled = candidate.map(point => ({
    x: candidateCenter.x + (point.x - candidateCenter.x) * scaleX,
    y: candidateCenter.y + (point.y - candidateCenter.y) * scaleY,
  }));
  const scaledBounds = boundsOf(scaled);
  const scaledCenter = centroid(scaled);
  const dx = referenceCenter.x - scaledCenter.x;
  const dy = anchor === "bottom"
    ? referenceBounds.maxY - scaledBounds.maxY
    : referenceCenter.y - scaledCenter.y;

  return scaled.map(point => ({ x: point.x + dx, y: point.y + dy }));
}

function deriveProfileNoseTarget(centerNose: Point[], quarterNose: Point[], rawProfileNose: Point[]): Point[] {
  const alignedQuarter = alignByAnchor(centerNose, quarterNose, "top");
  const alignedProfile = alignByAnchor(centerNose, rawProfileNose, "top");
  const fittedProfile = fitContourToReference(alignedQuarter, alignedProfile, 0.45, 0.72);
  return lerpPoints(alignedQuarter, fittedProfile, 0.72);
}

function stabilizeHeadTarget(reference: Point[], candidate: Point[], widthFit: number): Point[] {
  return fitContourToReference(reference, candidate, widthFit, 0.92, "bottom");
}

function stabilizeEarTarget(reference: Point[], candidate: Point[]): Point[] {
  return fitContourToReference(reference, candidate, 0.78, 0.86, "center");
}

export function buildRig(side: 1 | -1, samples: number): MorphPair[] {
  const mirrored = side < 0;
  const centerFace = sampleSelector(SOURCE.centerFace, samples, false);
  const angleFace = sampleSelector(SOURCE.angleFace, samples, mirrored);
  const profileFace = sampleSelector(SOURCE.profileFace, samples, mirrored);
  const backFace = sampleSelector(SOURCE.backFace, samples, false);
  const centerNose = sampleSelector(SOURCE.centerNose, samples, false);
  const angleNose = sampleSelector(SOURCE.angleNose, samples, mirrored);
  const targetNose = sampleSelector(SOURCE.profileNose, samples, mirrored);

  return [
    {
      key: "face",
      source: centerFace.points,
      quarter: stabilizeHeadTarget(centerFace.points, alignByAnchor(centerFace.points, angleFace.points, "bottom"), 0.45),
      target: stabilizeHeadTarget(centerFace.points, alignByAnchor(centerFace.points, profileFace.points, "bottom"), 0.30),
      back: stabilizeHeadTarget(centerFace.points, alignByAnchor(centerFace.points, backFace.points, "bottom"), 0.68),
    },
    {
      key: "nose",
      source: centerNose.points,
      quarter: alignByAnchor(centerNose.points, angleNose.points, "top"),
      target: deriveProfileNoseTarget(centerNose.points, angleNose.points, targetNose.points),
    },
  ];
}

export function buildEarRig(samples: number): Record<EarKey, EarRig> {
  const angleLeft = sampleSelector(SOURCE.angleEar, samples, false);
  const angleRight = sampleSelector(SOURCE.angleEar, samples, true);
  const profileLeft = sampleSelector(SOURCE.profileEar, samples, false);
  const profileRight = sampleSelector(SOURCE.profileEar, samples, true);
  const rig = {} as Record<EarKey, EarRig>;

  for (const [key, def] of Object.entries(EAR_DEFS) as [EarKey, (typeof EAR_DEFS)[EarKey]][]) {
    const front = sampleSelector(def.frontSelector, samples, false);
    const back = sampleSelector(def.backSelector, samples, false);
    rig[key] = {
      key,
      localSide: def.localSide,
      front: front.points,
      quarterLeft: stabilizeEarTarget(front.points, alignByAnchor(front.points, angleLeft.points, "top")),
      profileLeft: stabilizeEarTarget(front.points, alignByAnchor(front.points, profileLeft.points, "top")),
      back: stabilizeEarTarget(front.points, alignByAnchor(front.points, back.points, "top")),
      quarterRight: stabilizeEarTarget(front.points, alignByAnchor(front.points, angleRight.points, "top")),
      profileRight: stabilizeEarTarget(front.points, alignByAnchor(front.points, profileRight.points, "top")),
    };
  }

  return rig;
}

export function computeEarState(key: EarKey, localSide: 1 | -1, yawRad: number): EarState {
  const s = Math.sin(yawRad);
  const c = Math.cos(yawRad);
  const profile = Math.abs(s);
  const back = Math.max(0, -c);
  const depth = localSide * s;
  const screenX = PIVOT_X + localSide * 118 * c;
  const isNear = depth > 0.035;
  const isFar = depth < -0.035;
  const frontLike = Math.abs(depth) <= 0.035;
  const fusion = isNear
    ? 0.18 + 0.34 * smoothstep(remapClamp(profile, 0.62, 1.0))
    : smoothstep(remapClamp(profile, 0.18, 0.72));

  return {
    key,
    localSide,
    depth,
    screenX,
    isNear,
    isFar,
    frontLike,
    profileT: smoothstep(profile),
    backT: smoothstep(back),
    fusion: clamp(fusion, 0, 1),
  };
}

export function computeEarPoints(earRig: EarRig, yawDeg: number): Point[] {
  const deg = ((yawDeg % 360) + 360) % 360;
  return interpolatePointKeyframes(
    [
      { t: 0, points: earRig.front },
      { t: 45, points: earRig.quarterLeft },
      { t: 90, points: earRig.profileLeft },
      { t: 180, points: earRig.back },
      { t: 270, points: earRig.profileRight },
      { t: 315, points: earRig.quarterRight },
      { t: 360, points: earRig.front },
    ],
    deg,
    0.42,
  );
}

export function computeRigViewState(yawRad: number, yawDeg: number): ViewState {
  const s = Math.sin(yawRad);
  const c = Math.cos(yawRad);
  const side: 1 | -1 = s >= 0 ? 1 : -1;
  const profile = Math.abs(s);
  const back = Math.max(0, -c);

  let zone: ViewState["zone"];
  if (back >= 0.82) zone = "BACK_PROXY";
  else if (back >= 0.45) zone = "REAR_TRANSITION";
  else if (profile >= 0.82) zone = "PROFILE";
  else if (profile >= 0.28) zone = "THREE_QUARTER";
  else zone = "FRONT";

  const t = smoothstep(profile);
  const noseFusion = zone === "BACK_PROXY" ? 1 : smoothstep(remapClamp(profile, 0.28, 0.68));
  const earRight = computeEarState("earRight", 1, yawRad);
  const earLeft = computeEarState("earLeft", -1, yawRad);
  const ears = [earRight, earLeft];
  const depthSorted = [...ears].sort((a, b) => a.depth - b.depth || a.localSide - b.localSide);
  const farEar = depthSorted[0];
  const nearEar = depthSorted[1];

  const showNose = zone !== "BACK_PROXY" && !(zone === "REAR_TRANSITION" && back >= 0.68);
  const showMouth = zone === "FRONT" || zone === "THREE_QUARTER";

  let noseMode: ViewState["noseMode"];
  if (!showNose) noseMode = "HIDDEN";
  else if (noseFusion < 0.22) noseMode = "SEPARATE";
  else if (noseFusion < 0.82) noseMode = "MERGING";
  else noseMode = "FUSED";

  return {
    yawDeg,
    side,
    profile,
    back,
    zone,
    t,
    noseFusion,
    noseMode,
    ears,
    earRight,
    earLeft,
    nearEar,
    farEar,
    showNose,
    showMouth,
  };
}
