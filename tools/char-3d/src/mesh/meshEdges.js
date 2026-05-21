export function buildEdges(faces) {
  const map = new Map();
  for (let fi=0; fi<faces.length; fi++) {
    const vs = faces[fi].v;
    for (let i=0;i<3;i++) {
      const a=vs[i], b=vs[(i+1)%3];
      const lo=Math.min(a,b), hi=Math.max(a,b);
      const key=`${lo}_${hi}`;
      if (!map.has(key)) map.set(key, {a:lo,b:hi,faces:[]});
      map.get(key).faces.push(fi);
    }
  }
  return [...map.values()];
}
