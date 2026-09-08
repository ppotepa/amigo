# Contributions

The plugin registers `amigo.gfx.npr-playground.extractor` and `gfx.npr@1`.
Its render service publishes NprDrawCommand packets and one shared paper
background. The runtime bundle bridge copies these neutral commands into WGPU
frame packets. SceneColor and SceneDepth are the declared outputs; no renderer
object-name heuristic or Mesh3D scene entity is required.

RuntimeControlProvider exposes the complete settings tree and a selected-object
alias. PresetProvider validates a candidate before replacing all settings. Panel
actions use the existing Rhai event queue, not backend-specific entrypoints.

The plugin contributes an `NprDrawCommand` and does not call a backend directly.

The workshop adds `appearance.*`, object rotation switches, read-only badge/history
state and `stats.*` metadata. `npr-look` contributes appearance-only preset storage;
`npr-playground` retains complete scene storage. Panel tabs, pinned groups, reset
requests and choice artwork are neutral panel-api contracts, reusable by an editor.

The appearance metadata includes `tool`, gesture confidence/simplification/
correction/overstroke, tool pressure/hardness, nib angle/aspect, paper
tooth/grain, ink dryness and optional tone hatching controls. Rhai only sends
typed values through the provider; it does not own projection, line selection or
tessellation policy.
