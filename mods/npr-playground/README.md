# NPR Playground

This mod is the first small slice of the rebuilt NPR implementation. It renders
one Suzanne model. It starts centred and rotates continuously; the playground
controls its position, manual rotation and orbital camera. Minimal Ink and the
first Pencil Animation profile are available. Pencil Animation deliberately
starts with paper and selective graphite contour lines, without fill or a hatch
grid; tonal layers will be added one at a time.

Scene: `suzanne`.

Run a scene with:

```powershell
cargo run -p amigo-launcher -- --hosted --mod=npr-playground --scene=suzanne
```

Controls: arrows rotate the model, left drag orbits, mouse wheel zooms, right
drag moves Suzanne, Shift + right drag moves it vertically, R resets the model,
Home resets the camera, 1 selects Perspective, and 2 selects Orthographic.
Press 3 for Minimal Ink or 4 for Pencil Animation. Space pauses/resumes rotation.
