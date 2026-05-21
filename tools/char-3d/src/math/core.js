export const TAU = Math.PI * 2;
export const EPS = 1e-7;

export const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
export const clamp01 = v => clamp(v, 0, 1);
export const lerp = (a, b, t) => a + (b - a) * t;
export const deg = v => v * Math.PI / 180;
export const fmt = (n, d = 2) => Number(n).toFixed(d);

export function v3(x = 0, y = 0, z = 0) { return { x, y, z }; }
export function sub(a, b) { return { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }; }
export function cross(a, b) { return { x: a.y*b.z - a.z*b.y, y: a.z*b.x - a.x*b.z, z: a.x*b.y - a.y*b.x }; }
export function dot(a, b) { return a.x*b.x + a.y*b.y + a.z*b.z; }
export function len(a) { return Math.hypot(a.x, a.y, a.z); }
export function norm(a) { const l = len(a) || 1; return { x: a.x/l, y: a.y/l, z: a.z/l }; }
export function len2(a) { return Math.hypot(a.x, a.y); }
export function norm2(a) { const l = len2(a) || 1; return { x: a.x/l, y: a.y/l }; }
export function rot2(v, a) { const c = Math.cos(a), s = Math.sin(a); return { x: v.x*c - v.y*s, y: v.x*s + v.y*c }; }
export function mix2(a, b, t) { return norm2({ x: a.x*(1 - t) + b.x*t, y: a.y*(1 - t) + b.y*t }); }
export function triArea2(a, b, c) { return Math.abs((b.sx - a.sx)*(c.sy - a.sy) - (b.sy - a.sy)*(c.sx - a.sx)) * 0.5; }

export function hash01(n) {
  const x = Math.sin(n * 12.9898 + 78.233) * 43758.5453123;
  return x - Math.floor(x);
}

export function noise(seed, i = 0) {
  return hash01(seed + i * 37.719) * 2 - 1;
}

export function bary2(p, a, b, c) {
  const den = (b.sy - c.sy)*(a.sx - c.sx) + (c.sx - b.sx)*(a.sy - c.sy);
  if (Math.abs(den) < EPS) return null;
  const u = ((b.sy - c.sy)*(p.x - c.sx) + (c.sx - b.sx)*(p.y - c.sy)) / den;
  const v = ((c.sy - a.sy)*(p.x - c.sx) + (a.sx - c.sx)*(p.y - c.sy)) / den;
  const w = 1 - u - v;
  return { u, v, w };
}

export function baryInside(b, pad = 0.01) {
  return b && b.u >= -pad && b.v >= -pad && b.w >= -pad;
}

export function mixPoint(a, b, c, u, v, w) {
  return { x: a.sx*u + b.sx*v + c.sx*w, y: a.sy*u + b.sy*v + c.sy*w, z: a.z*u + b.z*v + c.z*w };
}

export function pointFromBary(f, u, v, w) {
  const p = f.p;
  return { x: p[0].sx*u + p[1].sx*v + p[2].sx*w, y: p[0].sy*u + p[1].sy*v + p[2].sy*w, z: p[0].z*u + p[1].z*v + p[2].z*w };
}
