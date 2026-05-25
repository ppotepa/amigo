# Light 2D Pipeline

Light 2D owns semantic 2D lighting state: global lights, light groups, and
lightmap sources.

## Flow
- Scene commands queue global light, light group, and lightmap source data.
- Lighting scene services store active commands for the current scene.
- Runtime extraction bridges convert lighting state into render packets and
  lighting targets.

## Targets
- Writes `SceneLighting`.
- Contributes `SceneHighlight` and `SceneEmissive`.
- The plugin is not a direct renderable source; it declares lighting intent for
  backend execution.
