# NprPlayground

## Layer-stack migration

The current look patch uses one `layers` list rather than separate `paint` and
`stroke` collections. Each entry retains its stable `layer_id`, explicit `order`
and complete `parameters`. Save emits only this shape. The Layers panel exposes
brushes, masks, grouping, build-up and variants. Brush editing is staged as a
draft; Apply is one mutation and Cancel discards it. The host provides a
reusable child editor window without another playground session.

Preview an existing document from the repository root:

```powershell
rtk cargo run -p amigo-npr-playground-plugin --example migrate_layer_stack -- mods/npr-playground/scenes/cube/npr.scene.yml
```

Add `--apply` immediately after `--` to apply. Multiple explicit file paths are
accepted. The tool previews all files before writing, then saves each document
atomically using the existing SHA conflict checks. Before each write it creates
an exclusive, durable `<document>.before-layer-stack` backup; it never overwrites
a backup. This is not a multi-file transaction: a later I/O failure may leave
earlier documents migrated with their backups. Re-running a migrated document is
a no-op. Scene object look overrides and model look defaults are also supported.
The tool preserves layer order, all parameters and paper; YAML formatting and
mapping-key order may change. Runtime does not accept the split format.

Scene profiles also persist named variants and the pinned brush library. Built-in
brushes are copied into the profile as read-only definitions; `SaveBrushVersion`
adds the next numeric version explicitly. Existing references retain their old
version until `UpdateBrushVersion` is dispatched.

The mod ships a `drawing-studio-demo` preset based on `comic-ink`. It exercises
the complete ordered composition: watercolour underpainting, surface hatching,
form lines and contour ink.

## Current application: Basic Mode

The shipped client is deliberately a small drawing workspace, not an NPR
authoring mock-up. It has exactly three visible decisions:

```text
Models -> Drawing viewport -> Looks
```

`Models` selects one source model. Selecting another model automatically keeps
the changed drawing as a local, model-bound draft and opens a clean document.
`Looks` contains only the curated `comic-ink`, `pencil-study`, and
`watercolour-wash` illustration recipes. Choosing one applies it immediately.
The top bar provides Save and Reset view; orbit, pan, zoom and selection remain
direct viewport interactions.

Look cards receive an asynchronous `look_previews` image generated from the
same resolved Look, layer order, fill triangles and tessellated strokes as NPR
rendering. The preview queue has a hard concurrency limit of two. A card says
`Rendering…` until its real image arrives; it never substitutes a decorative
stroke for renderer output.

Basic Mode does not expose a layer stack, geometry sources, masks, blend modes,
brush versions, Brush Editor, diagnostics, variants, draft browser, motion or
presentation controls. Those domain features remain serialized and renderable,
but are intentionally not part of the first user workflow. `drawing-studio-demo`
is an authoring fixture and is not selectable in Basic Mode.

The future expansion path is explicit rather than tabbed into the initial UI:

```text
Basic: choose model and finished look
Studio: customize a copy of a look as an ordered layer composition
Brush Lab: preview and version a procedural brush
```

Brush Lab must render its future stroke, hatch and surface samples through the
same NPR implementation as the document renderer. Static glyphs are not brush
previews and must not be presented as such.

`asset_browser::brush_thumbnail` now provides that renderer-side primitive for
the five built-ins. It resolves Ink against `Silhouette`, Graphite against
`FormLines`, Hatching against `ShadowHatch`, Flat Fill against `FlatFill`, and
Watercolour Wash against `Wash`; the pinned `BrushDefinition` is installed in
an isolated preview library before packet extraction. The future Brush Lab UI
must consume this value rather than reproduce these media in CSS or canvas.
The companion snapshot publishes these cached values as `brush_previews`, keyed
by `brush-id@version`, through a separate two-item asynchronous queue. Basic
Mode deliberately ignores this field; Brush Lab is its only consumer.

## Runtime and transport

Gallery opts in with `playgrounds: [{ id: npr-playground, auto_open: true }]`.
Cube has no companion declaration and opens the normal Winit game window.
Gallery opens only Tauri: its engine ticks without a primary Winit window or a
second renderer. Closing Tauri also exits the engine. The host chooses this
lifecycle from the scene's automatic companion declaration, not its scene name.
Closing the companion retains the scene; entering another scene ends its process
and loopback WebSocket session. Reopening requires entering the scene again.

`playground-api` owns neutral descriptors, versioned snapshots/deltas, action
envelopes and viewport requests. `NprPlaygroundService` owns validation, settings,
selection, authoring history and saves. Runtime bundles own process composition,
transport and the offscreen render bridge. The neutral Tauri host embeds the
domain's Svelte client. The app only dispatches the generic client mode.

The child receives a length-prefixed bootstrap over piped stdin. The host binds
an ephemeral loopback port and checks protocol, playground ID, exact WebView
origin and a single-use random token. CSP permits that session endpoint only.
The presentation selector above the image offers Native GPU, Local RGBA and
JPEG. First launch requests Native GPU; subsequent launches restore the last
successfully presented mode. An unavailable mode reports its reason and requires
manual selection. A failed switch preserves the previous active presentation.
Presentation and camera interaction preferences do not dirty the scene.

On Windows, Native GPU uses a separate DX12 device and shared textures/fences;
scenes using a Winit window retain their own backend. Local RGBA uses three read-only
WebView2 SharedBuffers and a reusable WebGL2 texture; the presenter releases stale
generation buffers and disposes the texture/program when the session ends. JPEG uses binary WebSocket
frames and libjpeg-turbo Q92. The native pipe authenticates the specific companion
PID with a separate stdin credential that never enters JavaScript. The protocol
v2 frame header carries generation, sequence, revision, dimensions, format and
processed input ID. At most two frames remain outstanding through consumption.
Native GPU slots additionally wait for the consumer GPU fence.

The desktop Basic layout has a models column, one large paper viewport and a
looks column. On narrow windows Models becomes a drawer and Looks becomes a
horizontal rail below the viewport. There are no inspector tabs. Primary-button
Select and Orbit modes are available in the compact viewport toolbar; right
drag orbits, middle drag pans and wheel zoom remain available in every mode.

Camera navigation sums relative updates while the previous request is processed.
A shared controller handles canvas and native input in CSS coordinates, with
picking scaled into the render image. Drag threshold is crossed once; pointer
capture, cancellation and lost focus finish the gesture. All domain mutations
share one queue and require the matching request ID and state revision. One
camera gesture creates one undo entry. A conflict refreshes state and discards
the rejected gesture's remaining movement. The animation toggle changes `selected.rotating`
property and remains disabled until an object is selected.

The HUD reports actual presentation, resolution and FPS averaged over 500 ms.
Diagnostics exposes extraction, submit, readback and encoding timings. Spinning
is capped at 30 FPS; interaction and camera settling at 60 FPS. After drawing
variation, spinning and fade stop, the worker produces no continuous frames.
Adaptive resolution is a local opt-in setting (off by default); capture bypasses
its scale. Native child surfaces are hidden under modal dialogs and mobile docks.

Scene data lives in `scenes/<scene>/npr.scene.yml`; reusable looks live in
`npr/looks/<id>.npr-look.yml`. Save All writes the scene's session changes,
Save Look updates the active look while retaining includes, and Save As writes
a standalone resolved look. Hash conflicts reject overwrite. Reload discards
session edits; Save As preserves the effective look under a new ID. Successful
saves clear undo/redo history. Dock/tab preferences are local client state.

Local imports use the native picker and host-side GLB/glTF validation. Complete
artifacts are published atomically under `assets/models/<id>` with an
`npr-model.json` manifest for subsequent discovery. `AssetCatalog` owns async
Loading/Ready/Failed states. `npr/catalogs.yml` lists trusted HTTP(S) manifests;
each artifact file has a path, byte count and SHA-256. Verified cache files live
under `.cache/npr`; failed downloads never publish a partial model.
Optional `assets/models/<id>/npr.model.yml` documents declare model look defaults
below included/active looks and scene/object overrides.

`npr_playground_metadata()` and `npr_playground_dispatch(intent)` use the same
service through the generic Rhai adapter. Selection, model, look and save events
are published to the script event queue. The frontend never evaluates Rhai.

Build prerequisites: the repository Rust toolchain, Node/npm, and the native
Tauri prerequisites. Run `npm ci` in `playground-client` once. Cargo builds the
frontend when the runtime enables `playground-client`; generated dist assets
are embedded from Cargo's output directory and are not committed.

Use `rtk cargo run --profile playground -p amigo-app -- --hosted --mod npr-playground --scene gallery`
for optimized interactive validation. The profile inherits release, includes
debug symbols and enables incremental compilation (the repository's filesystem
configuration can override incremental caching). Building libjpeg-turbo requires
CMake and NASM. Native GPU requires Windows/DX12 texture and fence sharing on the
same adapter; Local RGBA requires WebView2 SharedBuffer and WebGL2. macOS/Linux
native presentation remains future work; the JPEG transport is platform-neutral.
