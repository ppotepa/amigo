export function createEmptyBounds() {
  return {
    minX: Infinity,
    minY: Infinity,
    minZ: Infinity,
    maxX: -Infinity,
    maxY: -Infinity,
    maxZ: -Infinity,
    cx: 0,
    cy: 0,
    cz: 0,
    radius: 0,
  };
}

export function expandBoundsByPoint(bounds, x, y, z) {
  bounds.minX = Math.min(bounds.minX, x);
  bounds.minY = Math.min(bounds.minY, y);
  bounds.minZ = Math.min(bounds.minZ, z);
  bounds.maxX = Math.max(bounds.maxX, x);
  bounds.maxY = Math.max(bounds.maxY, y);
  bounds.maxZ = Math.max(bounds.maxZ, z);
  return bounds;
}

export function expandBoundsByTriangle(bounds, ax, ay, az, bx, by, bz, cx, cy, cz) {
  expandBoundsByPoint(bounds, ax, ay, az);
  expandBoundsByPoint(bounds, bx, by, bz);
  expandBoundsByPoint(bounds, cx, cy, cz);
  return bounds;
}

export function finalizeBounds(bounds) {
  if (!Number.isFinite(bounds.minX)) {
    bounds.minX = bounds.minY = bounds.minZ = 0;
    bounds.maxX = bounds.maxY = bounds.maxZ = 0;
  }
  bounds.cx = (bounds.minX + bounds.maxX) * 0.5;
  bounds.cy = (bounds.minY + bounds.maxY) * 0.5;
  bounds.cz = (bounds.minZ + bounds.maxZ) * 0.5;
  const dx = bounds.maxX - bounds.cx;
  const dy = bounds.maxY - bounds.cy;
  const dz = bounds.maxZ - bounds.cz;
  bounds.radius = Math.hypot(dx, dy, dz);
  return bounds;
}

export function boundsCorners(bounds) {
  return [
    [bounds.minX, bounds.minY, bounds.minZ],
    [bounds.maxX, bounds.minY, bounds.minZ],
    [bounds.minX, bounds.maxY, bounds.minZ],
    [bounds.maxX, bounds.maxY, bounds.minZ],
    [bounds.minX, bounds.minY, bounds.maxZ],
    [bounds.maxX, bounds.minY, bounds.maxZ],
    [bounds.minX, bounds.maxY, bounds.maxZ],
    [bounds.maxX, bounds.maxY, bounds.maxZ],
  ];
}

export function boundsVolume(bounds) {
  return Math.max(0, bounds.maxX - bounds.minX)
    * Math.max(0, bounds.maxY - bounds.minY)
    * Math.max(0, bounds.maxZ - bounds.minZ);
}

export function boundsDiagonal(bounds) {
  return Math.hypot(
    bounds.maxX - bounds.minX,
    bounds.maxY - bounds.minY,
    bounds.maxZ - bounds.minZ,
  );
}

export function intersectsViewport(bounds2d, viewport, margin = 0) {
  return bounds2d.maxX >= -margin
    && bounds2d.maxY >= -margin
    && bounds2d.minX <= viewport.width + margin
    && bounds2d.minY <= viewport.height + margin;
}

export function area2d(bounds2d) {
  return Math.max(0, bounds2d.maxX - bounds2d.minX) * Math.max(0, bounds2d.maxY - bounds2d.minY);
}
