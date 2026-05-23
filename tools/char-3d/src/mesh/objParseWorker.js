import { parseOBJ } from './objParser.js';
import { meshRuntimeTransferList, prepareMeshRuntime } from './meshRuntime.js';

self.addEventListener('message', event => {
  parseMessage(event.data || {});
});

async function parseMessage(message) {
  const { id, text, url, file, name } = message;
  try {
    let source = '';
    if (typeof text === 'string') source = text;
    else if (file && typeof file.text === 'function') source = await file.text();
    else if (url) {
      const response = await fetch(url);
      if (!response.ok) throw new Error(`${name || url} fetch failed: ${response.status}`);
      source = await response.text();
    }
    const mesh = parseOBJ(source);
    mesh.name = name || mesh.name;
    const runtime = prepareMeshRuntime(mesh);
    mesh.verts = [];
    mesh.faces = [];
    mesh.edges = [];
    self.postMessage({ id, ok: true, mesh, sourceLength: source.length }, meshRuntimeTransferList(runtime));
  } catch (error) {
    self.postMessage({
      id,
      ok: false,
      error: error && error.message ? error.message : String(error),
    });
  }
}
