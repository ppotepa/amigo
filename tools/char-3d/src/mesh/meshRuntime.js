export function prepareMeshRuntime(mesh) {
  if (!mesh) return null;
  const hasFaces = Array.isArray(mesh.faces) && mesh.faces.length > 0;
  const hasEdges = Array.isArray(mesh.edges) && mesh.edges.length > 0;
  const hasVerts = Array.isArray(mesh.verts) && mesh.verts.length > 0;
  if (mesh.runtime && (!hasVerts || !hasFaces || !hasEdges || (
    mesh.runtime.vertCount === mesh.verts.length
    && mesh.runtime.faceCount === mesh.faces.length
    && mesh.runtime.edgeCount === mesh.edges.length
  ))) {
    return mesh.runtime;
  }
  if (!hasVerts || !hasFaces || !hasEdges) return null;

  const vertCount = mesh.verts.length;
  const vertX = new Float32Array(vertCount);
  const vertY = new Float32Array(vertCount);
  const vertZ = new Float32Array(vertCount);
  for (let i = 0; i < vertCount; i++) {
    const p = mesh.verts[i];
    vertX[i] = p.x;
    vertY[i] = p.y;
    vertZ[i] = p.z;
  }

  const faceCount = mesh.faces.length;
  const faceA = new Int32Array(faceCount);
  const faceB = new Int32Array(faceCount);
  const faceC = new Int32Array(faceCount);
  for (let i = 0; i < faceCount; i++) {
    const v = mesh.faces[i].v;
    faceA[i] = v[0];
    faceB[i] = v[1];
    faceC[i] = v[2];
  }

  const edgeCount = mesh.edges.length;
  const edgeA = new Int32Array(edgeCount);
  const edgeB = new Int32Array(edgeCount);
  const edgeF0 = new Int32Array(edgeCount);
  const edgeF1 = new Int32Array(edgeCount);
  for (let i = 0; i < edgeCount; i++) {
    const edge = mesh.edges[i];
    edgeA[i] = edge.a;
    edgeB[i] = edge.b;
    edgeF0[i] = edge.faces[0] ?? -1;
    edgeF1[i] = edge.faces[1] ?? -1;
  }

  mesh.runtime = { vertCount, vertX, vertY, vertZ, faceCount, faceA, faceB, faceC, edgeCount, edgeA, edgeB, edgeF0, edgeF1 };
  return mesh.runtime;
}

export function syncMeshRuntimeVertices(mesh) {
  const runtime = prepareMeshRuntime(mesh);
  if (!runtime || !Array.isArray(mesh?.verts)) return runtime;
  for (let i = 0; i < runtime.vertCount; i++) {
    const p = mesh.verts[i];
    runtime.vertX[i] = p.x;
    runtime.vertY[i] = p.y;
    runtime.vertZ[i] = p.z;
  }
  return runtime;
}

export function meshRuntimeTransferList(runtime) {
  if (!runtime) return [];
  return [
    runtime.vertX.buffer,
    runtime.vertY.buffer,
    runtime.vertZ.buffer,
    runtime.faceA.buffer,
    runtime.faceB.buffer,
    runtime.faceC.buffer,
    runtime.edgeA.buffer,
    runtime.edgeB.buffer,
    runtime.edgeF0.buffer,
    runtime.edgeF1.buffer,
  ];
}
