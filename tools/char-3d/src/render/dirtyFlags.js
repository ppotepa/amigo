export const DIRTY_FLAGS = Object.freeze({
  MESH: 'mesh',
  PROJECTION: 'projection',
  VISIBILITY: 'visibility',
  NPR: 'npr',
  PAINT: 'paint',
  DISPLAY: 'display',
});

export function createDirtyFlags() {
  return {
    mesh: true,
    projection: true,
    visibility: true,
    npr: true,
    paint: true,
    display: true,
    last: 'initial',
  };
}

export function markDirty(flags, scope = DIRTY_FLAGS.PROJECTION) {
  flags.last = scope;
  if (scope === DIRTY_FLAGS.MESH) {
    flags.mesh = true;
    flags.projection = true;
    flags.visibility = true;
    flags.npr = true;
    flags.paint = true;
    flags.display = true;
    return;
  }
  if (scope === DIRTY_FLAGS.PROJECTION) {
    flags.projection = true;
    flags.visibility = true;
    flags.npr = true;
    flags.paint = true;
    flags.display = true;
    return;
  }
  if (scope === DIRTY_FLAGS.VISIBILITY) {
    flags.visibility = true;
    flags.npr = true;
    flags.paint = true;
    flags.display = true;
    return;
  }
  if (scope === DIRTY_FLAGS.NPR) {
    flags.npr = true;
    flags.display = true;
    return;
  }
  if (scope === DIRTY_FLAGS.PAINT) {
    flags.paint = true;
    flags.display = true;
    return;
  }
  flags.display = true;
}

export function clearDirty(flags) {
  flags.mesh = false;
  flags.projection = false;
  flags.visibility = false;
  flags.npr = false;
  flags.paint = false;
  flags.display = false;
}
