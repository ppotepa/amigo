import { extractFbxAdapterMesh } from './fbxAdapter.js';

function writeAscii(view, offset, text) {
  for (let i = 0; i < text.length; i++) view.setUint8(offset + i, text.charCodeAt(i));
}

export function buildFbxClipAmc(rt, options = {}) {
  const fps = options.fps || 60;
  const duration = options.duration || rt.duration || 1;
  const frameCount = Math.max(1, Math.ceil(duration * fps));

  if (rt.mixer) rt.mixer.setTime(0);
  const firstMesh = extractFbxAdapterMesh(rt);
  const vertexCount = firstMesh.verts.length;
  const faceCount = firstMesh.faces.length;

  const headerBytes = 32;
  const faceBytes = faceCount * 3 * 4;
  const vertexBytes = frameCount * vertexCount * 3 * 4;
  const buffer = new ArrayBuffer(headerBytes + faceBytes + vertexBytes);
  const view = new DataView(buffer);

  writeAscii(view, 0, 'AMC1');
  view.setUint32(4, 1, true);
  view.setFloat32(8, fps, true);
  view.setFloat32(12, duration, true);
  view.setUint32(16, vertexCount, true);
  view.setUint32(20, faceCount, true);
  view.setUint32(24, frameCount, true);
  view.setUint32(28, 0, true);

  let offset = headerBytes;
  for (const face of firstMesh.faces) {
    view.setUint32(offset, face.v[0], true); offset += 4;
    view.setUint32(offset, face.v[1], true); offset += 4;
    view.setUint32(offset, face.v[2], true); offset += 4;
  }

  for (let frame = 0; frame < frameCount; frame++) {
    const t = Math.min(duration, frame / fps);
    if (rt.mixer) rt.mixer.setTime(t);
    const mesh = extractFbxAdapterMesh(rt);
    if (mesh.verts.length !== vertexCount) {
      throw new Error(`FBX clip bake failed: vertex count changed at frame ${frame}`);
    }
    for (const p of mesh.verts) {
      view.setFloat32(offset, p.x, true); offset += 4;
      view.setFloat32(offset, p.y, true); offset += 4;
      view.setFloat32(offset, p.z, true); offset += 4;
    }
  }

  return buffer;
}

export function downloadArrayBuffer(filename, mime, buffer) {
  const blob = new Blob([buffer], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 750);
}
