# amigo-3d-mesh

3D mesh scene service for authored geometry references.

## Responsibility

- Store mesh asset bindings.
- Store entity transforms for mesh draw commands.
- Register 3D mesh capability.
- Import geometry-only glTF/GLB, including hierarchical transforms, morph targets
  and skinning with all joint/weight sets.
- Expose named animation clips and node/property tracks; sample STEP, LINEAR and
  CUBICSPLINE curves. Playback clocks, looping and UI belong to consumers.

`load_gltf_geometry` returns the normalized bind pose plus immutable animation
data. `sample_animation(clip_index, seconds)` applies morphs, then skin/node
transforms, then the **same** bind-pose normalization. Topology is welded once
using deformation signatures; it is not regenerated per frame.

Buffers may be embedded in GLB, base64 data URIs, or relative files confined to
the model directory. Invalid accessors, hierarchies, skin weights and channels
return explicit errors. The importer accepts triangle primitives; it does not
decode materials/textures or animation extensions outside core glTF.

Interpolation and deformation follow the [glTF 2.0 specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#animations).

Validation: `cargo test -p amigo-3d-mesh` includes generated glTF and GLB fixtures
for hierarchy, fixed normalization, morphs, skinning and malformed input.

## Not here

- GPU upload.
- Material shading.

## Depends on

- amigo-assets.
- amigo-core.
- amigo-math.
- amigo-render-api.
- amigo-capabilities.
- amigo-runtime.
- amigo-scene.
