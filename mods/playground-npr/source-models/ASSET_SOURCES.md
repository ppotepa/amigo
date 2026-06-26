# NPR Source Models

This directory stores raw model inputs for the NPR mesh-cache workbench.
The WGPU 3D path imports these GLB files into a cached geometry buffer for the
current comic-line NPR workbench.

Included files:

- `threejs/Soldier.glb` from Three.js Examples, used as the primary character target for projected-stroke NPR.
- `khronos/BoxTextured.glb` from Khronos glTF Sample Models, used as a static hard-edge topology reference.
- `khronos/Fox.glb` from Khronos glTF Sample Models, used as an animated/skinned topology reference.
- `khronos/Box.glb` from Khronos glTF Sample Models, used as a minimal hard-edge reference.
- `khronos/BoxInterleaved.glb` from Khronos glTF Sample Models, used to verify interleaved GLB buffers.
- `khronos/BoxAnimated.glb` from Khronos glTF Sample Models, used as an animated-source static topology reference.
- `khronos/CesiumMan.glb` from Khronos glTF Sample Models, used as a character silhouette reference.

License notes:

- `Soldier.glb`: Three.js examples asset; keep provider and source URL attached in the manifest.
- `BoxTextured.glb`: donated by Cesium for glTF testing, Creative Commons Attribution 4.0.
- `Fox.glb`: base mesh by PixelMannen under CC0; rigging and animation by @tomkranis under CC-BY 4.0.
- `Box.glb`, `BoxInterleaved.glb`, `BoxAnimated.glb`: Khronos sample assets for conformance and renderer testing.
- `CesiumMan.glb`: Khronos sample asset for character renderer testing.

The local `npr.html` prototype also contains an embedded OBJ hand model. It is
referenced in `manifest.yml`, but it is not copied here because its source and
license need to be made explicit before committing it as a reusable mod asset.
