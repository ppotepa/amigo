# NPR Source Models

This directory stores raw model inputs for the NPR mesh-cache workbench.
The current WGPU 3D path still renders procedural debug meshes from `Mesh3D`
commands, so these files are staged inputs for the future importer/cache.

Included files:

- `khronos/BoxTextured.glb` from Khronos glTF Sample Models, used as a static hard-edge topology reference.
- `khronos/Fox.glb` from Khronos glTF Sample Models, used as an animated/skinned topology reference.

License notes:

- `BoxTextured.glb`: donated by Cesium for glTF testing, Creative Commons Attribution 4.0.
- `Fox.glb`: base mesh by PixelMannen under CC0; rigging and animation by @tomkranis under CC-BY 4.0.

The local `npr.html` prototype also contains an embedded OBJ hand model. It is
referenced in `manifest.yml`, but it is not copied here because its source and
license need to be made explicit before committing it as a reusable mod asset.
