import { PIVOT_X, type Contour, type EarKey, type EarRig, type EarState, type MorphPair, type Point, type ViewState } from "./types";
import {
  centroid,
  clamp,
  distanceSq,
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
  profileFace: "#LEFT_PROFILE #FACE_PROFILE",
  profileNose: "#LEFT_PROFILE #NOSE-PROFILE",
  profileEar: "#LEFT_PROFILE #EAR_LEFT",
  backFace: "#BACK path:nth-of-type(1)",
  backLeftEar: "#BACK path:nth-of-type(2)",
  backRightEar: "#BACK path:nth-of-type(3)",
} as const;

const EAR_DEFS: Record<EarKey, { localSide: 1 | -1; frontSelector: string; backSelector: string; label: string }> = {
  earRight: { localSide: 1, frontSelector: SOURCE.centerRightEar, backSelector: SOURCE.backLeftEar, label: "right" },
  earLeft: { localSide: -1, frontSelector: SOURCE.centerLeftEar, backSelector: SOURCE.backRightEar, label: "left" },
};

function samplePath(path: SVGPathElement, count: number, mirrored = false): Contour {
  const total = path.getTotalLength();
  let points: Point[] = [];
  for (let i = 0; i < count; i++) {
    const point = path.getPointAtLength((i / count) * total);
    points.push({ x: point.x, y: point.y });
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

export function buildRig(side: 1 | -1, samples: number): MorphPair[] {
  const mirrored = side < 0;
  const centerFace = sampleSelector(SOURCE.centerFace, samples, false);
  const profileFace = sampleSelector(SOURCE.profileFace, samples, mirrored);
  const backFace = sampleSelector(SOURCE.backFace, samples, false);
  const centerNose = sampleSelector(SOURCE.centerNose, samples, false);
  const targetNose = sampleSelector(SOURCE.profileNose, samples, mirrored);

  return [
    {
      key: "face",
      source: centerFace.points,
      target: alignTargetToSource(centerFace.points, profileFace.points),
      back: alignTargetToSource(centerFace.points, backFace.points),
    },
    {
      key: "nose",
      source: centerNose.points,
      target: alignTargetToSource(centerNose.points, targetNose.points),
    },
  ];
}

export function buildEarRig(samples: number): Record<EarKey, EarRig> {
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
      profileLeft: alignTargetToSource(front.points, profileLeft.points),
      back: alignTargetToSource(front.points, back.points),
      profileRight: alignTargetToSource(front.points, profileRight.points),
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
  let a: Point[];
  let b: Point[];
  let t: number;

  if (deg <= 90) {
    a = earRig.front;
    b = earRig.profileLeft;
    t = smoothstep(deg / 90);
  } else if (deg <= 180) {
    a = earRig.profileLeft;
    b = earRig.back;
    t = smoothstep((deg - 90) / 90);
  } else if (deg <= 270) {
    a = earRig.back;
    b = earRig.profileRight;
    t = smoothstep((deg - 180) / 90);
  } else {
    a = earRig.profileRight;
    b = earRig.front;
    t = smoothstep((deg - 270) / 90);
  }

  return lerpPoints(a, b, t);
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

  let noseMode: ViewState["noseMode"];
  if (zone === "BACK_PROXY") noseMode = "HIDDEN";
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
    showNose: zone !== "BACK_PROXY",
    showMouth: zone !== "BACK_PROXY",
  };
}
