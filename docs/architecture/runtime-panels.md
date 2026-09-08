# Runtime scene panels

Scenes opt in with top-level `panels: [{id, layout, auto_open}]`. Paths are relative
to `scene.yml`, confined to the owning mod. Normal scenes create no extra window.
Interactive startup uses `--hosted`; offscreen/headless sessions never spawn a UI.

The engine contract is reusable by an editor:

```text
scene panel reference + external YAML layout
  -> amigo-panel-api (existing SceneUiNode document)
  -> amigo-panels (lifecycle, validation, command queue)
  -> RuntimeControlService (typed provider metadata and values)
  -> domain provider
```

`amigo-panel-egui` is one consumer of this contract, not the owner of scene state.
Bundles connect it to the thin app entrypoint. Each panel uses a child process of
the current executable (`--runtime-panel-client`, internal only). stdin/stdout are
private length-prefixed JSON pipes, capped at 4 MiB. Logs use stderr. Protocol
version, scene generation, layout revision and ordered request IDs reject stale
traffic. Bounded queues and replaceable 30 Hz snapshots keep a slow UI from
blocking simulation. Closing/crashing a panel does not terminate the scene;
scene changes drop the old connections. The parent owns child cleanup.

## Authoring and Rhai

`value_bind`, `text_bind`, `enabled_bind`, `visible_bind` reference runtime-control
paths. Values support bool, integer, floating point, string/asset reference,
Vec2/Vec3 and RGBA. Provider access/range checks apply to UI and scripts alike;
layout constraints can narrow the accepted range. Unknown paths, invalid widget
types and unsupported curve editors report errors rather than guessing behavior.

`on_click` and `on_change` publish existing scene script events. `on_change`
appends the accepted property's path to the declared payload; scripts can read its
typed value. Rhai exposes `world.controls.get/set/reset`, `world.panels.open/close`
and `world.presets.save(domain,name,overwrite)/load/reset/list`.

Panel commands drain in PreUpdate; domain systems update in Update; rendering
publishes its immutable packet in RenderExtract. Panel snapshots are throttled
independently and may trail simulation by one tick. Optimistic UI edits are kept
until an acknowledged snapshot arrives. Layout hot reload uses the engine's
HotReloadService with an isolated watch set, preserves domain state and stable
control IDs, and retains the last valid document on errors.

## Presets

PresetProvider owns complete validation and atomic replacement of its domain
state. Files are versioned YAML under `.amigo/presets/<mod>/<scene>/`, written via
a synced temporary file and rename. Names cannot traverse directories; overwrite
is explicit. `preset_name_bind` enables the saved-name chooser, and
`confirm_actions` lists actions requiring native confirmation. No window position
or measured FPS is persisted. Script preset errors return to the panel.

Examples: `mods/npr-playground` (gallery, styles, camera, animation) and
`mods/panel-playground` (independent RenderLayer2D controls). No standalone editor,
network listener, runtime shader authoring or in-process egui overlay is introduced.

Validation: `cargo test -p amigo-panel-api`, `cargo test -p amigo-panels`,
`cargo test -p amigo-npr-playground-plugin`, and the app's targeted NPR golden test.
The egui host requires Rust 1.92 or newer (eframe 0.35 / WGPU 29).

## Workshop presentation contract

`presentation` maps stable control IDs to optional `tooltip`, `suffix`, `reset`,
`collapsed`, `pin: top|bottom`, and `choices` hints. Pins apply only to direct root
children. Tab IDs must correspond one-to-one to child page IDs. Ordinary unpinned
panels retain their scrolling layout.

Choice entries declare `value`, `label`, optional `artwork_bind` and `status_bind`.
Fixed choices may use an `artwork` key instead; `navigation: true` enables prev/next.
All bindings participate in the same validated, batched metadata snapshot.
`artwork` maps keys to painter-ordered triangles with normalized `[0,1]` coordinates
and RGB8 colors. The UI clips a single mesh per thumbnail; it never imports models,
projects geometry or chooses a domain style. Artwork describes a neutral reference,
not an implicit promise of a live viewport preview.

Reset requests carry the same epoch, ordering and visibility checks as edits and
call the provider's reset operation. `preset_domain_bind` optionally filters saved
names; overwrites cannot replace another domain's preset. NPR's bounded history
and before/after policy belong to its provider, not to the egui process.

## Host startup and troubleshooting

Both `amigo-app` and `amigo-launcher` dispatch the internal panel-client mode
through runtime bundles **before** config loading, TUI startup or stdout logging.
Dev launcher profiles host the runtime in-process, so their panel child is another
launcher process. Release profiles run the release app, whose panel child is another
release app. stdout is reserved for the framed protocol in either client.

Rebuild after pulling changes, then run one of:

```text
cargo run -p amigo-app -- --hosted --mod npr-playground --scene gallery
cargo run -p amigo-launcher -- --hosted --profile dev --mod npr-playground --scene gallery
cargo build --release -p amigo-app
cargo run -p amigo-launcher -- --hosted --profile release --mod npr-playground --scene gallery
```

The TUI's Gallery selection uses the same hosted path. Headless/offscreen runs
and scenes without panel declarations create no panel processes. Close a running
launcher before rebuilding on Windows. An old release binary must be rebuilt;
the launcher never silently substitutes a debug build.

Spawn failures, early exits/crashes, malformed transport, incompatible protocol and
a missing handshake after five seconds report `[panels.host]` to stderr, the
runtime run log and the engine dev console. Repeated frames do not repeat the same
diagnostic or auto-respawn failed children. The scene keeps running. After resolving
the error, reopen explicitly with `world.panels.open("npr")` in its Rhai context.
Normal window close is signalled through the protocol and is not reported as a
crash. Scene changes dispose of old processes and reject stale generations.

Regression commands:

```text
cargo test -p amigo-panels
cargo test -p amigo-app --test panel_entrypoint --test panel_lifecycle
cargo test -p amigo-launcher --test panel_entrypoint
cargo test -p amigo-app npr_playground_offscreen_matches_reviewed_golden
```

The executable tests run outside the repository directory to catch accidental
config/TUI initialization before the handshake. Native lifecycle coverage runs on
Windows; framed protocol and engine policy tests are platform-independent.
