const STAGES = ['selection', 'projection', 'depth', 'visibility', 'contours', 'marks', 'paint', 'draw'];

export function createPerfStats() {
  return {
    frames: 0,
    cacheHits: 0,
    cacheMisses: 0,
    lastCacheHit: false,
    lastTotalMs: 0,
    last: Object.fromEntries(STAGES.map(stage => [stage, 0])),
    counters: {},
  };
}

export function resetPerfFrame(stats, cacheHit = false) {
  stats.frames++;
  stats.lastCacheHit = cacheHit;
  stats.lastTotalMs = 0;
  for (const stage of STAGES) stats.last[stage] = 0;
  stats.counters = {};
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

export function setPerfCounter(stats, name, value) {
  stats.counters[name] = value;
}

export function formatPerfStats(stats, fmt) {
  const hitRate = stats.frames ? Math.round((stats.cacheHits / Math.max(1, stats.cacheHits + stats.cacheMisses)) * 100) : 0;
  const c = stats.counters || {};
  const selection = (c.sceneUnitsTotal || c.selectedFaces || c.facesSelected)
    ? `<br>selection units ${c.sceneUnitsVisible || 0}/${c.sceneUnitsTotal || 0} · verts ${c.vertsSelected || c.selectedVertices || 0} · faces ${c.facesSelected || c.selectedFaces || 0}/${c.facesTotal || 0} · edges ${c.edgesSelected || c.selectedEdges || 0}`
    : '';
  const detail = (c.detailTier0Units || c.detailTier1Units || c.detailTier2Units || c.detailTier3Units || c.detailTier4Units)
    ? `<br>detail D0 ${c.detailTier0Units || 0} · D1 ${c.detailTier1Units || 0} · D2 ${c.detailTier2Units || 0} · D3 ${c.detailTier3Units || 0} · D4 ${c.detailTier4Units || 0}`
    : '';
  const counts = c.facesTotal
    ? `<br>faces ${c.facesVisible || 0}/${c.facesOnScreen || 0}/${c.facesTotal || 0} · depth ${c.facesDepth || 0} · contours ${c.contoursDrawn || 0}/${c.contoursTested || 0} · marks ${c.marksGenerated || 0}/${c.marksBudget || 0} · paint ${c.paintFaces || 0}/${c.paintRegions || 0}`
    : '';
  const worker = c.workerMs
    ? `<br>worker ${fmt(c.workerMs, 1)}ms · dropped ${c.workerDropped || 0}${c.workerFallback ? ' · fallback' : ''}`
    : '';
  return `perf: <b>${fmt(stats.lastTotalMs, 1)}ms</b> · cache: <b>${stats.lastCacheHit ? 'hit' : 'miss'}</b>/${hitRate}%<br>` +
    `selection ${fmt(stats.last.selection, 1)} · projection ${fmt(stats.last.projection, 1)} · depth ${fmt(stats.last.depth, 1)} · visibility ${fmt(stats.last.visibility, 1)} · contours ${fmt(stats.last.contours, 1)} · marks ${fmt(stats.last.marks, 1)} · paint ${fmt(stats.last.paint, 1)} · draw ${fmt(stats.last.draw, 1)} ms${selection}${detail}${counts}${worker}`;
}
