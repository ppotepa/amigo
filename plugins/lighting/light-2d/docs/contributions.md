# Light 2D Contributions

Light 2D emits explicit lighting and camera optics intent.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- Global light and light group state for `SceneLighting`.
- Lightmap source metadata with explicit source references and channel data.

## Optical Adapter
`light_to_camera_optics_source` maps a light source to an emissive material
optical source with `camera.fx_source` role.

Lightmap existence alone is not optical intent; target routing must come from a
declared contribution or role.
