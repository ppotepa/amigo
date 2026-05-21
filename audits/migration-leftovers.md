# Migration leftovers

## Remaining migration themes

1. PostFX descriptor registry.
2. WGPU PostFX pipeline registry.
3. Camera debug policy descriptors.
4. Cached-image PostFX descriptors.
5. Camera optical coverage adapter registry.
6. Component metadata provider migration.
7. Plugin waterfall docs and tests.
8. Explicit Rotten Club light group camera responses/contributions.

## Suggested order

```text
1. PostFX audit and descriptors.
2. Descriptor metadata for current effects.
3. Pilot registry with CameraOptics, FocusBlur, RainGlass.
4. Debug policy migration.
5. WGPU pipeline registry.
6. Scene metadata provider cleanup.
7. Plugin docs/test strengthening.
```

## Do not combine

Do not combine PostFX registry migration with scene metadata cleanup or mod content authoring. These are separate risk areas.
