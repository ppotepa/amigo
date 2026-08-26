# Render golden tests

`amigo-render-api::compare_golden_rgba8` is the backend-neutral comparison primitive for deterministic offscreen captures. Golden suites should render a fixed scene, fixed viewport and fixed seed, read back RGBA8 pixels, and compare them against a checked-in or artifact-provided reference.

Use exact comparison for deterministic CPU/reference paths. GPU suites may declare a small explicit channel/pixel tolerance when cross-driver quantization makes byte-exact comparison inappropriate. Never hide broad visual changes behind large tolerances.

Recommended coverage set: sprite alpha/composition, 2D lighting, bloom/camera optics, particles, UI overlay, and representative 3D material/text/mesh scenes.
