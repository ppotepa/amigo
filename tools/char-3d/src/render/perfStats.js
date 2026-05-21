const STAGES = ['projection', 'depth', 'visibility', 'contours', 'marks', 'paint', 'draw'];

export function createPerfStats() {
  return {
    frames: 0,
    cacheHits: 0,
    cacheMisses: 0,
    lastCacheHit: false,
    lastTotalMs: 0,
    last: Object.fromEntries(STAGES.map(stage => [stage, 0])),
  };
}

export function resetPerfFrame(stats, cacheHit = false) {
  stats.frames++;
  stats.lastCacheHit = cacheHit;
  stats.lastTotalMs = 0;
  for (const stage of STAGES) stats.last[stage] = 0;
}

export function markCacheHit(stats) {
  stats.cacheHits++;
  stats.lastCacheHit = true;
}

export function markCacheMiss(stats) {
  stats.cacheMisses++;
  stats.lastCacheHit = false;
}

export function timeSection(stats, name, fn) {
  const t0 = performance.now();
  const value = fn();
  stats.last[name] = (stats.last[name] || 0) + performance.now() - t0;
  return value;
}

export function timeSectionEnd(stats, name, t0) {
  stats.last[name] = (stats.last[name] || 0) + performance.now() - t0;
}

export function finishPerfFrame(stats, t0) {
  stats.lastTotalMs = performance.now() - t0;
}

export function formatPerfStats(stats, fmt) {
  const hitRate = stats.frames ? Math.round((stats.cacheHits / Math.max(1, stats.cacheHits + stats.cacheMisses)) * 100) : 0;
  return `perf: <b>${fmt(stats.lastTotalMs, 1)}ms</b> · cache: <b>${stats.lastCacheHit ? 'hit' : 'miss'}</b>/${hitRate}%<br>` +
    `projection ${fmt(stats.last.projection, 1)} · depth ${fmt(stats.last.depth, 1)} · visibility ${fmt(stats.last.visibility, 1)} · contours ${fmt(stats.last.contours, 1)} · marks ${fmt(stats.last.marks, 1)} · paint ${fmt(stats.last.paint, 1)} · draw ${fmt(stats.last.draw, 1)} ms`;
}
