import { buildEdges } from './meshEdges.js';

function v3(x = 0, y = 0, z = 0) { return { x, y, z }; }

export function parseOBJ(text) {
  const verts = [];
  const faces = [];
  let lineStart = 0;
  while (lineStart <= text.length) {
    let lineEnd = text.indexOf('\n', lineStart);
    if (lineEnd < 0) lineEnd = text.length;
    const raw = text.slice(lineStart, lineEnd);
    lineStart = lineEnd + 1;
    const line = raw.trim();
    if (!line || line[0] === '#') continue;
    const parts = line.split(/\s+/);
    if (parts[0] === 'v' && parts.length >= 4) {
      verts.push(v3(Number(parts[1]), Number(parts[2]), Number(parts[3])));
    } else if (parts[0] === 'f' && parts.length >= 4) {
      const ids = [];
      for (let i=1;i<parts.length;i++) {
        let id = parseInt(parts[i].split('/')[0], 10);
        if (!Number.isFinite(id)) continue;
        if (id < 0) id = verts.length + id + 1;
        ids.push(id - 1);
      }
      for (let i=1;i<ids.length-1;i++) {
        if (ids[0] !== ids[i] && ids[i] !== ids[i+1]) faces.push({v:[ids[0], ids[i], ids[i+1]], id:faces.length});
      }
    }
  }
  if (!verts.length || !faces.length) throw new Error('OBJ parser: nie znaleziono v/f.');
  let min=v3(Infinity,Infinity,Infinity), max=v3(-Infinity,-Infinity,-Infinity);
  for (const p of verts) {
    min.x=Math.min(min.x,p.x); min.y=Math.min(min.y,p.y); min.z=Math.min(min.z,p.z);
    max.x=Math.max(max.x,p.x); max.y=Math.max(max.y,p.y); max.z=Math.max(max.z,p.z);
  }
  const center=v3((min.x+max.x)/2,(min.y+max.y)/2,(min.z+max.z)/2);
  const scale = 2 / Math.max(max.x-min.x, max.y-min.y, max.z-min.z);
  const nverts = verts.map(p => v3((p.x-center.x)*scale, (p.y-center.y)*scale, (p.z-center.z)*scale));
  return { verts:nverts, faces, edges:buildEdges(faces), name:'Suzanne/Susan OBJ', sourceType:'obj' };
}
