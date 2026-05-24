import { cross, deg, noise, norm, v3 } from '../math/core.js';

export function createProjectionContext(p) {
  const ctx = {
    width: p.width,
    height: p.height,
    controlMode: p.controlMode,
    projectionMode: p.projectionMode,
    yaw: p.yaw,
    pitch: p.pitch,
    zoom: p.zoom,
    cameraYaw: p.cameraYaw,
    cameraPitch: p.cameraPitch,
    cameraX: p.cameraX,
    cameraY: p.cameraY,
    cameraZ: p.cameraZ,
    focalLength: p.focalLength || 35,
    cameraDollyScale: p.cameraDollyScale || 1,
    projectionWobble: p.projectionWobble || 0,
    randomSeed: p.randomSeed || 0,
    centerX: p.centerX ?? p.width / 2,
    centerY: p.centerY ?? p.height / 2,
    sourceScaleMul: p.sourceScaleMul ?? 1,
    sourceWobbleMul: p.sourceWobbleMul ?? 1,
  };

  if (ctx.controlMode === 'freelook') {
    const yawRad = deg(ctx.cameraYaw);
    const pitchRad = deg(ctx.cameraPitch);
    ctx.fwd = norm(v3(
      Math.sin(yawRad) * Math.cos(pitchRad),
      -Math.sin(pitchRad),
      -Math.cos(yawRad) * Math.cos(pitchRad),
    ));
    ctx.rgt = norm(v3(Math.cos(yawRad), 0, Math.sin(yawRad)));
    ctx.upV = norm(cross(ctx.rgt, ctx.fwd));
    ctx.scale = Math.min(ctx.width, ctx.height) * 0.1;
    return ctx;
  }

  const yaw = deg(ctx.yaw);
  const pitch = deg(ctx.pitch);
  const cameraYaw = deg(-ctx.cameraYaw);
  const cameraPitch = deg(-ctx.cameraPitch);
  ctx.cy = Math.cos(yaw);
  ctx.sy = Math.sin(yaw);
  ctx.cp = Math.cos(pitch);
  ctx.sp = Math.sin(pitch);
  ctx.ccy = Math.cos(cameraYaw);
  ctx.csy = Math.sin(cameraYaw);
  ctx.ccp = Math.cos(cameraPitch);
  ctx.csp = Math.sin(cameraPitch);
  ctx.scale = Math.min(ctx.width, ctx.height) * 0.36 * ctx.zoom * ctx.cameraDollyScale * ctx.sourceScaleMul;
  return ctx;
}

export function projectWorldPoint(ctx, px, py, pz, index = 0, out = {}) {
  if (ctx.controlMode === 'freelook') {
    const rx = px - ctx.cameraX;
    const ry = py - ctx.cameraY;
    const rz = pz - ctx.cameraZ;
    const cx = rx * ctx.rgt.x + ry * ctx.rgt.y + rz * ctx.rgt.z;
    const cy = rx * ctx.upV.x + ry * ctx.upV.y + rz * ctx.upV.z;
    const cz = rx * ctx.fwd.x + ry * ctx.fwd.y + rz * ctx.fwd.z;
    const perspective = ctx.projectionMode === 'perspective'
      ? ctx.focalLength / Math.max(0.1, cz)
      : ctx.focalLength / 10.0;
    let sx = ctx.centerX + cx * ctx.scale * perspective;
    let sy = ctx.centerY - cy * ctx.scale * perspective;
    if (ctx.projectionWobble > 0 && index >= 0) {
      const seed = (index + 1) * 409.17 + ctx.randomSeed * 23.91;
      sx += noise(seed, 1) * ctx.projectionWobble * ctx.sourceWobbleMul;
      sy += noise(seed, 2) * ctx.projectionWobble * ctx.sourceWobbleMul;
    }
    out.x = cx;
    out.y = cy;
    out.z = cz;
    out.sx = sx;
    out.sy = sy;
    out.inFront = cz >= 0.1;
    return out;
  }

  const x1 = px * ctx.cy + pz * ctx.sy;
  const z1 = -px * ctx.sy + pz * ctx.cy;
  const y2 = py * ctx.cp - z1 * ctx.sp;
  const z2 = py * ctx.sp + z1 * ctx.cp;
  const vx = x1 - ctx.cameraX;
  const vy = y2 - ctx.cameraY;
  const vz = z2 - ctx.cameraZ;
  const x3 = vx * ctx.ccy + vz * ctx.csy;
  const z3 = -vx * ctx.csy + vz * ctx.ccy;
  const y4 = vy * ctx.ccp - z3 * ctx.csp;
  const z4 = vy * ctx.csp + z3 * ctx.ccp;
  let sx = ctx.centerX + x3 * ctx.scale;
  let sy = ctx.centerY - y4 * ctx.scale;
  if (ctx.projectionWobble > 0 && index >= 0) {
    const seed = (index + 1) * 409.17 + ctx.randomSeed * 23.91;
    sx += noise(seed, 1) * ctx.projectionWobble * ctx.sourceWobbleMul;
    sy += noise(seed, 2) * ctx.projectionWobble * ctx.sourceWobbleMul;
  }
  out.x = x3;
  out.y = y4;
  out.z = z4;
  out.sx = sx;
  out.sy = sy;
  out.inFront = true;
  return out;
}
