# Office Desk Visual Maps

These maps were extracted from the standalone HTML mockup and are kept as technical authoring assets for the future material/source-buffer pipeline.

Runtime MVP uses only:

- `layered-images/office-desk/base_albedo.png`
- `depth-maps/office-desk-depth/depth.png`
- `Camera2D` depth of field
- `BeaconLight2D` glow/light sprites

Do not bind these maps directly to final rendering until the engine has dedicated semantics for them:

- `surface_mask.png`: R = reflectivity, G = roughness, B = glass/transmission response, A = surface/effect mask.
- `depth_aux_rgba.png`: R = auxiliary depth-like data, G = local height/protrusion, B = occluder strength, A = valid/effect mask.
- `material_id.png`: segmentation/material ID map. This is not color texture data.
- `_rejected/emissive_rejected.png`: rejected mockup emissive map. Do not use as emissive, highlight, glow, bokeh source, or light map.

MVP rule: keep these files available for authoring and future engine work, but do not map them to `visual_maps.emissive`, `visual_maps.highlight`, or `visual_maps.wetness`.
