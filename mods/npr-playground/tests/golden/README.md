# NPR cube golden

`cube-512.png` is a reviewed WGPU offscreen reference: 512x512, rotations
rx=0.36 / ry=0.71 radians, seed 42, default ComicInk, paused animation.

Test: `cargo test -p amigo-app npr_playground_offscreen_matches_reviewed_golden`.
It checks domain statistics as well as the stored image. Up to 512 differing
pixels (0.2%) are allowed for rasterization differences at edges; full color
changes, missing geometry and systematic hidden-line regressions fail.

Explicit regeneration only: set `AMIGO_UPDATE_NPR_GOLDEN=1`, run that test,
inspect the PNG, then rerun without the variable. Missing references fail rather
than silently approving a new render.
