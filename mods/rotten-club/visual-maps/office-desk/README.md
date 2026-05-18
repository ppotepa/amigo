# Office Desk visual maps

Source of truth: `index (19).html` from `index (19).zip`.

No new/generated graphics were created for this package. All PNGs in this
folder are extracted from base64 `data:image/png;base64,...` constants inside
the HTML mockup.

## Active runtime maps

- `base_albedo.png` from `BASE_IMAGE_URL` is the LayeredImage2D base plate.
- `depth.png` from `DEPTH_IMAGE_URL` remains the DepthMap2D texture used by
  camera/DOF and as the preferred plate relight depth source when available.
- `depth_aux_rgba.png` from `AUX_IMAGE_URL` and `surface_mask.png` from
  `SURFACE_IMAGE_URL` are used only by the plate_relight MVP. They are data
  maps for local pseudo-normal, occlusion, reflectivity, roughness, glass, and
  effect mask response.
- `material_id.png` is still not used at runtime.
- `_source/emissive_rejected_from_html.png` is still not used at runtime.
- Do not connect these technical maps to `visual_maps.emissive`,
  `visual_maps.highlight`, or `visual_maps.wetness`.

## Technical maps extracted from HTML

- `surface_mask.png` from `SURFACE_IMAGE_URL`
  - intended channel meaning from the mockup:
    - R = reflectivity
    - G = roughness
    - B = glass/transmission/material response
    - A = surface/effect mask
  - IMPORTANT: this is not a wetness map.

- `depth_aux_rgba.png` from `AUX_IMAGE_URL`
  - intended channel meaning from the mockup:
    - R = auxiliary/camera depth-like channel
    - G = local height / protrusion
    - B = occluder / shadow-caster strength
    - A = valid/effect mask

## Runtime debug

- `D` in office-desk cycles plate relight debug views.
- `R` returns to `final_output`.
- Aux debug views read `depth_aux_rgba.png`.
- Surface debug views read `surface_mask.png`.
- `effective_depth`, `normal`, `occlusion`, and `contribution` are derived in
  `PLATE_RELIGHT_SHADER`.
- `plate_relight_shadow` is derived in `PLATE_RELIGHT_SHADER` from effective
  depth, `depth_aux_rgba.png` B/A, and the cursor beacon position.
- `plate_relight_light_mask`, `plate_relight_ndl`,
  `plate_relight_specular`, `plate_relight_material_gate`, and
  `plate_relight_lit_raw` separate light attenuation, normal response,
  specular response, material gating, and pre-camera relit plate output.
- `plate_relight_shadow` is not a shadow map texture and does not use
  `material_id.png`.
- `material_id.png` remains unused.
- `_source/emissive_rejected_from_html.png` remains audit-only.
- These maps are not `visual_maps.normal`, `visual_maps.wetness`,
  `visual_maps.highlight`, or `visual_maps.emissive`.

- `material_id.png` from `MATERIAL_IMAGE_URL`
  - intended as a material/object segmentation map.
  - Do not use it as a visible overlay.
  - Do not feed it into final albedo, emissive, bokeh, or highlight paths.

## Rejected map

- `_source/emissive_rejected_from_html.png` was extracted from
  `EMISSIVE_IMAGE_URL` for audit only.
- It is intentionally not referenced by any scene YAML file.
- Do not connect it to `visual_maps.emissive`; it was explicitly rejected for
  this iteration.

## Extraction manifest

`_source/extraction_manifest.json` records the source constants, output paths,
image sizes, and SHA-256 hashes for traceability.
