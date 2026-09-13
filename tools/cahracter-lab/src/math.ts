import { type Bounds, PIVOT_X, PIVOT_Y, type Point } from "./types";

export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

export function smoothstep(t: number): number {
  t = clamp(t, 0, 1);
  return t * t * (3 - 2 * t);
}

export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function remapClamp(x: number, a: number, b: number): number {
  return clamp((x - a) / (b - a), 0, 1);
}

export function pointLerp(a: Point, b: Point, t: number): Point {
  return { x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t) };
}

export function lerpPoints(a: Point[], b: Point[], t: number): Point[] {
  return a.map((point, index) => pointLerp(point, b[index], t));
}

export type PointKeyframe = {
  t: number;
  points: Point[];
};

function keyframeTangent(frames: PointKeyframe[], frameIndex: number, pointIndex: number): Point {
  const previous = frames[Math.max(0, frameIndex - 1)];
  const next = frames[Math.min(frames.length - 1, frameIndex + 1)];
  const dt = next.t - previous.t || 1;
  return {
    x: (next.points[pointIndex].x - previous.points[pointIndex].x) / dt,
    y: (next.points[pointIndex].y - previous.points[pointIndex].y) / dt,
  };
}

export function interpolatePointKeyframes(frames: PointKeyframe[], value: number, curve = 0.55): Point[] {
  if (!frames.length) return [];
  if (value <= frames[0].t) return frames[0].points;
  if (value >= frames[frames.length - 1].t) return frames[frames.length - 1].points;

  const segmentIndex = Math.max(0, frames.findIndex((frame, index) => index > 0 && value <= frame.t) - 1);
  const a = frames[segmentIndex];
  const b = frames[segmentIndex + 1];
  const dt = b.t - a.t || 1;
  const u = clamp((value - a.t) / dt, 0, 1);
  const u2 = u * u;
  const u3 = u2 * u;
  const h00 = 2 * u3 - 3 * u2 + 1;
  const h10 = u3 - 2 * u2 + u;
  const h01 = -2 * u3 + 3 * u2;
  const h11 = u3 - u2;

  return a.points.map((point, index) => {
    const next = b.points[index];
    const tangentA = keyframeTangent(frames, segmentIndex, index);
    const tangentB = keyframeTangent(frames, segmentIndex + 1, index);
    const hermite = {
      x: h00 * point.x + h10 * dt * tangentA.x + h01 * next.x + h11 * dt * tangentB.x,
      y: h00 * point.y + h10 * dt * tangentA.y + h01 * next.y + h11 * dt * tangentB.y,
    };
    return pointLerp(pointLerp(point, next, u), hermite, curve);
  });
}

export function distanceSq(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return dx * dx + dy * dy;
}

export function centroid(points: Point[]): Point {
  const sum = points.reduce((acc, point) => ({ x: acc.x + point.x, y: acc.y + point.y }), { x: 0, y: 0 });
  return { x: sum.x / points.length, y: sum.y / points.length };
}

export function signedArea(points: Point[]): number {
  let sum = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i];
    const b = points[(i + 1) % points.length];
    sum += a.x * b.y - b.x * a.y;
  }
  return sum * 0.5;
}

export function boundsOf(points: Point[]): Bounds {
  const xs = points.map(point => point.x);
  const ys = points.map(point => point.y);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  return { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY };
}

export function normalized(points: Point[]): Point[] {
  const bounds = boundsOf(points);
  const width = bounds.width || 1;
  const height = bounds.height || 1;
  return points.map(point => ({
    x: (point.x - bounds.minX) / width,
    y: (point.y - bounds.minY) / height,
  }));
}

export function rotatePoints(points: Point[], offset: number): Point[] {
  const length = points.length;
  const out = new Array<Point>(length);
  for (let i = 0; i < length; i++) out[i] = points[(i + offset) % length];
  return out;
}

export function reversePoints(points: Point[]): Point[] {
  return [...points].reverse();
}

export function mirrorPoints(points: Point[], pivotX = PIVOT_X): Point[] {
  return points.map(point => ({ x: pivotX * 2 - point.x, y: point.y }));
}

export function pointsToPath(points: Point[], smoothAmount: number): string {
  if (!points.length || points.length < 2) return "";
  if (smoothAmount <= 0.001) {
    let d = `M ${points[0].x.toFixed(2)} ${points[0].y.toFixed(2)}`;
    for (let i = 1; i < points.length; i++) {
      d += ` L ${points[i].x.toFixed(2)} ${points[i].y.toFixed(2)}`;
    }
    return `${d} Z`;
  }

  const k = smoothAmount / 6;
  let d = `M ${points[0].x.toFixed(2)} ${points[0].y.toFixed(2)}`;
  for (let i = 0; i < points.length; i++) {
    const p0 = points[(i - 1 + points.length) % points.length];
    const p1 = points[i];
    const p2 = points[(i + 1) % points.length];
    const p3 = points[(i + 2) % points.length];
    const c1 = { x: p1.x + (p2.x - p0.x) * k, y: p1.y + (p2.y - p0.y) * k };
    const c2 = { x: p2.x - (p3.x - p1.x) * k, y: p2.y - (p3.y - p1.y) * k };
    d += ` C ${c1.x.toFixed(2)} ${c1.y.toFixed(2)}, ${c2.x.toFixed(2)} ${c2.y.toFixed(2)}, ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`;
  }
  return `${d} Z`;
}

export function applyPseudoDepth(points: Point[], yawRad: number, t: number, depth: number): Point[] {
  if (depth <= 0) return points;
  const side = Math.sign(Math.sin(yawRad)) || 1;
  const squeeze = 1 - depth * 0.09 * t;
  const parallax = side * depth * 13 * t;
  const verticalLift = -depth * 3.5 * t * Math.cos(yawRad);
  return points.map(point => ({
    x: PIVOT_X + (point.x - PIVOT_X) * squeeze + parallax,
    y: PIVOT_Y + (point.y - PIVOT_Y) * (1 + depth * 0.012 * t) + verticalLift,
  }));
}
