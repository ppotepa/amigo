Poniżej pełny, zaktualizowany `AGENTS.md` pod obecną architekturę `mc44k`.

````md
# AGENTS.md

This file defines how Codex CLI agents must work in the Amigo repository.

The goal is not only to make changes, but to make changes with the smallest practical amount of token usage, file reading, command execution, and architectural disruption.

Amigo is a fresh project. Do not preserve legacy paths after migration. Evolve systems in place. Prefer clean final names over compatibility shims.

Current architecture direction:

```text
apps/app                 = thin host
domains                  = own their runtime capabilities
runtime/bundles          = compose domain/runtime bundles
engine/devtools          = owns dev console, debug overlay, diagnostics, debug commands
engine/render-api        = owns render contracts
engine/camera            = owns shared camera contracts/services
engine/editor-api        = placeholder editor contracts only
engine/editor-session    = placeholder editor session only
````

Do not build a full editor now. Do not create `apps/editor`. Do not move domain execution back into `apps/app`.

---

# Prime Directive

Use `amigo-codemap` first.

Do not scan the repository blindly. Do not read whole files when a symbol slice is enough. Do not run broad build/test commands when a targeted crate check is enough.

Every non-trivial task should follow this shape:

```text
understand scope
  -> use codemap to locate exact files/symbols
  -> inspect only relevant slices
  -> apply a small manual patch or validated raw ops
  -> verify only touched crates
  -> report concise result
```

Prefer the shortest codemap command that answers the question.

Use codemap for targeted navigation:

```text
brief
changes --compact
change-plan
open-set --why
trace
symbols
slice
range-for-symbol
verify-plan --changed
fallout
```

Do not open concat snapshots during normal work.

---

# Current Architectural Goal

The active goal is no longer to “save” the architecture.

The runtime-only refactor is already advanced. The goal now is:

```text
close remaining seams
prevent app-centric regression
keep editor work as placeholder API only
stabilize naming and boundaries
```

The most important remaining seams are:

```text
apps/app/src/bootstrap.rs
apps/app/src/render_runtime.rs
apps/app/src/scene_runtime/mod.rs
```

Treat these as migration seams, not as places to add new domain behavior.

---

# Hard Prohibitions

Do not use these by default:

```text
Get-Content <large file>
cat <large file>
type <large file>
rg across the entire repo as first discovery
fd/find across the entire repo as first discovery
opening concat-output.txt
reading generated snapshots
cargo check --workspace
cargo test --workspace
full git diff
manual file-by-file browsing
large rewrite without codemap plan
large patch based on broad import blocks
creating new app-side domain wiring
creating apps/editor
```

Forbidden unless explicitly requested:

```text
workspace-wide test/check
broad formatting of unrelated files
mass renames
compatibility layers
v2 systems
parallel duplicate systems
leaving legacy paths after migration
adding domain crates directly to apps/app
building real editor UI
adding egui/imgui/winit/wgpu dependencies to editor-api or editor-session
```

Do not solve architecture issues by moving behavior into `apps/app`.

---

# Required Start Sequence

Start every non-trivial task with:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"

& $cm brief
& $cm changes --compact --hide-generated --limit 20
```

Then use a task-specific plan:

```powershell
& $cm change-plan "<task summary>" --limit 20
& $cm open-set "<target architecture or feature area>" --why --limit 12
```

Examples:

```powershell
& $cm change-plan "close app bootstrap seam move host registrations behind runtime bundles" --limit 20
& $cm open-set "bootstrap runtime bundles register_full_runtime_capabilities devtools plugin" --why --limit 12
```

```powershell
& $cm change-plan "verify render composition layers and WGPU frame composition builder" --limit 20
& $cm open-set "WgpuFrameCompositionBuilder FrameCompositionPlan CompositionLayer RenderSpace" --why --limit 12
```

---

# Token-Saving Discovery Rules

## First use `trace`

Use `trace` for exact symbols, strings, commands, feature names, type names, and function names.

Good:

```powershell
& $cm trace "register_full_runtime_capabilities" --limit 20
& $cm trace "WgpuFrameCompositionBuilder" --limit 20
& $cm trace "CompositionLayer" --limit 20
& $cm trace "RenderSpace" --limit 20
& $cm trace "CameraBinding::main" --limit 20
& $cm trace "InspectorSchema::placeholder" --limit 20
```

Use multiple focused traces instead of one broad search.

Bad:

```powershell
rg render
```

Good:

```powershell
& $cm trace "FrameCompositionPlan" --limit 20
& $cm trace "WgpuFrameCompositionBuilder" --limit 20
& $cm trace "DebugOverlay" --limit 20
```

## Then use `open-set --why`

Use `open-set --why` to get the smallest useful workset.

```powershell
& $cm open-set "runtime bundles WGPU render extractor bridges composition layers" --why --limit 12
```

The `--why` output tells why each file matters. Do not open unrelated files.

## Then use `symbols`

Use `symbols` before reading a file.

```powershell
& $cm symbols --file crates/runtime/bundles/src/wgpu_render_extractors/composition.rs --metadata --limit 80
```

## Then use `slice --symbol`

Read only the symbol you need.

```powershell
& $cm slice crates/runtime/bundles/src/wgpu_render_extractors/composition.rs --symbol WgpuFrameCompositionBuilder
& $cm slice crates/engine/render-api/src/composition.rs --symbol FrameCompositionPlan
& $cm slice crates/engine/camera/src/lib.rs --symbol CameraBinding
```

## Use `range-for-symbol` only when necessary

Use it for exact range boundaries before deletion or targeted replacement.

```powershell
& $cm range-for-symbol crates/runtime/bundles/src/wgpu_render_extractors/composition.rs WgpuFrameCompositionBuilder
```

Do not manually guess line ranges when codemap can provide them.

---

# Cost Tiers

Prefer lower-cost operations.

## Tier 0: Always allowed

```text
codemap brief
codemap changes --compact
codemap trace
codemap open-set --why
codemap symbols
codemap slice --symbol
codemap range-for-symbol
codemap verify-plan --changed
codemap fallout
```

## Tier 1: Allowed after scoped discovery

```text
cargo check -p touched_crate
cargo test -p touched_crate exact_test_name
codemap impact
codemap anchors --write
codemap anchor-check
```

## Tier 2: Use only with clear need

```text
small targeted rg in one file or one directory
reading a short file fully
cargo test -p crate without exact test name
```

## Tier 3: Avoid unless explicitly requested

```text
cargo check --workspace
cargo test --workspace
full repo search
full file dumps
global format
broad diff
```

---

# Raw Ops Protocol

Prefer small manual patches when that is clearer.

If using raw ops, use the current raw ops commands:

```powershell
$ops = @'
ACTION: ...
FILE: ...
CONTENT:
...
END
'@

$ops | & $cm ops-raw-check --yaml
$ops | & $cm ops-raw-apply --yaml
```

Do not use the old workflow:

```text
ops-preview --raw
ops-check --raw
ops-apply --raw
ops-summary --raw
```

Do not use:

```text
--strict without expected_hash
large FIND blocks based on imports
large whitespace-sensitive replacements
ops-skeleton --out plan.yml
ops-check --from plan.yml
ops-apply --from plan.yml
content_from
content_root
.amigo/ops/*
```

Raw ops should be small and locator-friendly.

Good FIND blocks:

```text
single function name
single struct name
single exact field line
short stable code snippet
```

Bad FIND blocks:

```text
large import group
entire module body
long formatted blocks
generated text
```

---

# Verification Rules

After any patch:

```powershell
& $cm verify-plan --changed
```

Then run only checks for touched crates.

Examples:

```powershell
cargo check -p amigo-runtime-bundles 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-render-api 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-camera 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-editor-api 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-editor-session 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-devtools 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-app 2>&1 | & $cm fallout --limit 80
```

Targeted test example:

```powershell
cargo test -p amigo-app architecture 2>&1 | & $cm fallout --limit 80
```

Do not paste full compiler output. Always pipe through `fallout`.

Do not claim workspace tests passed unless they were actually run.

---

# When a Command Fails

Do not immediately broaden the search.

Use this sequence:

```text
1. Read the error.
2. Use fallout summary.
3. Trace the missing symbol/type.
4. Slice the exact failing symbol.
5. Patch the exact issue.
6. Re-run only the failed crate check/test.
```

Example:

```powershell
cargo check -p amigo-runtime-bundles 2>&1 | & $cm fallout --limit 80
& $cm trace "missing_symbol_name" --limit 20
& $cm open-set "missing_symbol_name usage" --why --limit 8
```

Stop if the failure is unrelated or would require a broad refactor.

---

# Required Report Style

Final response must be concise and factual:

```text
Changed:
- file/symbol
- file/symbol

Verified:
- command
- command

Notes:
- skipped verification and why
- remaining follow-up
```

Do not claim success without verification.

Do not say workspace checks passed unless they were actually run.

---

# Architecture Rules

## App Is a Thin Host

`crates/apps/app` owns only true host responsibilities:

```text
window/event loop
host startup
platform surface orchestration
top-level RuntimeSession launch
temporary migration seams
```

`apps/app` must not own reusable engine/domain behavior.

Do not add new domain-specific systems, handlers, render extractors, editor contracts, or debug command implementations to `apps/app`.

Current allowed seams in `apps/app` are transitional:

```text
bootstrap.rs
render_runtime.rs
scene_runtime/mod.rs
```

When touching these files, prefer reducing app responsibility over adding more.

## Domains Own Runtime Capabilities

Domains should register their own runtime capabilities.

Good:

```text
2d domain registers 2D scene/runtime/script/render capabilities
3d domain registers 3D scene/runtime/script/render capabilities
audio domain registers audio runtime capabilities
ui domain registers UI runtime capabilities
devtools registers debug/dev capabilities
camera registers camera services/contracts
```

Bad:

```text
apps/app manually knows every domain capability
apps/app registers domain handlers directly
apps/app owns domain execution wiring
apps/app adds a new domain-specific runtime plugin
```

## runtime/bundles Composes, It Does Not Become an App

`crates/runtime/bundles` may compose sets of capabilities.

It may own bridge code that combines runtime domains for a backend, for example WGPU extraction bridges.

It must not become a second app.

Allowed names:

```text
CoreRuntimeBundle
TwoDRuntimeBundle
ThreeDRuntimeBundle
AudioRuntimeBundle
PlatformRuntimeBundle
DevtoolsRuntimeBundle
FullRuntimeBundle
WgpuSprite2dRenderExtractorBridge
WgpuMesh3dRenderExtractorBridge
WgpuFrameCompositionBuilder
```

Forbidden names in bundles:

```text
App*RenderExtractor
AppFrameCompositionBuilder
AppRuntime*
AppScene*
```

Use `App*` only when the type truly belongs to the host application.

## Devtools Owns Debug Runtime Behavior

`engine/devtools` owns:

```text
dev console model/registry
debug overlay model/service
diagnostics commands
debug script commands
debug runtime plugin/capabilities
```

`apps/app` should not own dev console or debug overlay implementation.

At most, app may contain a temporary re-export or host integration seam.

## Render Contracts Live in render-api

Shared render contracts belong in `engine/render-api`.

Examples:

```text
RenderSpace
CompositionLayer
FrameCompositionPlan
RenderViewPlan
RenderPassPlan
```

Do not define reusable render contracts in `apps/app`.

## Camera Contracts Live in engine/camera

Shared camera behavior belongs in `engine/camera`.

Expected helpers:

```text
CameraBinding::main()
CameraBinding::none()
CameraService::camera_by_binding(...)
```

World layers should use `CameraBinding::main()` when appropriate.

Do not invent app-local camera binding concepts.

## Editor API Is Placeholder Only

`engine/editor-api` is only a placeholder contract crate for future editor capabilities.

Allowed:

```text
ComponentTypeId
InspectorSchema
PropertyDescriptor
ValidationProvider
GizmoProvider
AssetPickerProvider
EditorCapabilityDescriptor
minimal helper constructors
```

Expected helpers:

```text
ComponentTypeId::new
InspectorSchema::placeholder
InspectorSchema::with_field
PropertyDescriptor::text
PropertyDescriptor::number
PropertyDescriptor::asset
PropertyDescriptor::bool
PropertyDescriptor::vec2
PropertyDescriptor::vec3
```

Forbidden:

```text
egui
imgui
winit
wgpu
real editor UI
viewport UI
asset browser UI
undo/redo workflow implementation
apps/app dependency
```

## Editor Session Is Placeholder Only

`engine/editor-session` may define placeholder editor session state.

Allowed:

```text
EditorSession
selection state
document/session placeholder state
capability registry placeholder
comments explaining that this is not a full editor app
```

Forbidden:

```text
real editor application
egui/imgui UI
winit event loop
wgpu renderer
asset browser
viewport implementation
dependency on apps/app
```

Do not create `apps/editor`.

## Editor Capabilities in Domains Are Minimal

Domains may expose `editor_capability.rs` files.

These must remain minimal placeholders.

Allowed:

```text
stable id
component_type
minimal InspectorSchema
simple PropertyDescriptor fields
no real editor implementation
```

Expected domain placeholders include, as relevant:

```text
Sprite2D
Text2D
Mesh3D
Audio
UI
Camera
Devtools
```

Forbidden:

```text
real inspector UI
real viewport gizmo implementation
asset browser implementation
undo/redo workflow
UI framework dependency
```

---

# Render Composition Rules

Final render composition should be layered and explicit.

Expected concepts:

```text
RenderSpace
CompositionLayer
FrameCompositionPlan.layers
FrameCompositionPlan::default_legacy_layers
FrameCompositionPlan::sorted_layers
FrameCompositionPlan::layers_for_space
WgpuFrameCompositionBuilder
```

Expected layer ordering:

```text
World3D
World2D
Ui
DebugOverlay
```

Debug overlay must render after game/world/UI effects.

World layers may use camera binding. Debug overlay should not accidentally be affected by world post-fx.

Do not reintroduce final-state names like:

```text
AppFrameCompositionBuilder
AppRenderExtractorProvider
App*RenderExtractor
LegacyComposite
SplitPassExperimental
render.mode legacy/split
```

If legacy names appear only in documentation, update the documentation.

---

# Bootstrap Rules

`apps/app/src/bootstrap.rs` should move toward thin host startup.

Good direction:

```text
parse host options
construct RuntimeSession
call runtime/bundles helper
call devtools helper where needed
start host event loop
```

Bad direction:

```text
manually register every domain capability
manually register dev console runtime behavior
manually own scene/script/system domain plugins
add more domain crates to app Cargo.toml
```

If touching bootstrap, prefer moving registration behind:

```text
runtime/bundles helper
engine/devtools helper
engine/session helper
```

Do not add new domain wiring directly to bootstrap.

---

# Scene Runtime Rules

`apps/app/src/scene_runtime/mod.rs` is a migration seam.

It may temporarily orchestrate loading/hydration, but new reusable scene behavior should move toward engine/session/domain crates.

Do not add new domain-specific scene handlers to app.

Scene command handlers should live with owning domains or runtime capability crates.

---

# Script Runtime Rules

Script commands should be registered by owning domains or devtools.

Do not reintroduce app-side script command handler directories.

Debug script commands belong in `engine/devtools`.

---

# Runtime Bundles Rules

Use bundles to compose capability sets.

Good:

```text
register_core_runtime_capabilities
register_2d_runtime_capabilities
register_3d_runtime_capabilities
register_audio_runtime_capabilities
register_platform_runtime_capabilities
register_devtools_runtime_capabilities
register_full_runtime_capabilities
```

Bad:

```text
apps/app registers each domain manually
runtime/bundles defines App* types
runtime/bundles owns host window loop
runtime/bundles becomes an application shell
```

WGPU bridge code may exist in bundles if it is clearly named as backend bridge code:

```text
Wgpu*RenderExtractorBridge
WgpuFrameCompositionBuilder
```

---

# Architecture Tests

Keep and extend architecture tests when closing seams.

Expected app architecture tests should protect against:

```text
app-side scene handler directories
app-side script handler directories
app-side domain systems
direct domain dependencies in apps/app
App*RenderExtractor names in runtime/bundles
AppFrameCompositionBuilder name
UI framework dependencies in editor-api
UI framework dependencies in editor-session
apps/app dependency in editor-api/editor-session
missing RenderSpace/CompositionLayer usage in composition
editor_capability.rs without minimal InspectorSchema helpers
```

Do not weaken architecture tests to make a patch pass.

If a test fails because architecture changed intentionally, update the test to protect the new boundary, not to remove protection.

---

# Documentation Rules

Keep documentation aligned with current names.

Important architecture docs:

```text
docs/architecture/runtime-refactor-status.md
docs/architecture/runtime-bundles.md
docs/architecture/render-composition.md
docs/architecture/editor-api-placeholder.md
```

Do not leave stale references to:

```text
AppFrameCompositionBuilder
AppRenderExtractorProvider
App*RenderExtractor
apps/app owned dev console
apps/app owned debug overlay
apps/editor
full editor UI
```

When renaming architecture concepts, update docs in the same patch if the docs are directly affected.

---

# No v2 Systems

Do not create:

```text
RenderPipelineV2
SceneCompilerV2
EditorTargetV2
ParticleSystemV2
NewRenderer
LegacyRenderer
```

Evolve existing systems in place.

Temporary migration names must be removed before finalizing the task.

---

# Cleanup-As-We-Go

Compatibility paths are allowed only during a migration step.

After migration, delete:

```text
legacy entrypoints
old wrappers
fallback branches
temporary compatibility helpers
dead_code allowances
migration comments that no longer apply
```

Fresh project rule: no permanent legacy.

A refactor is complete only when:

```text
new path is used by all call-sites
old path is deleted
compatibility helper is deleted
temporary flags are deleted
diagnostics exist where relevant
tests/checks pass
names match final architecture
no v2/legacy/experimental remains as final state
```

---

# Engine / App / Domain / Editor / Mod Boundaries

## Engine owns

```text
runtime data contracts
service registries
scene compilation contracts
scene validation contracts
render contracts
camera contracts
diagnostics/certification contracts
asset reference semantics
editor-facing metadata contracts
```

## Domains own

```text
domain components
domain runtime capabilities
domain scene command handlers
domain script command handlers
domain systems
domain render extraction capabilities
minimal editor capability descriptors
```

## Runtime bundles own

```text
composition of domain capability sets
backend bridge composition
full runtime capability registration helper
```

## Devtools owns

```text
dev console
debug overlay
diagnostics commands
debug script commands
debug runtime capabilities
```

## App owns

```text
host startup
window/event loop
surface orchestration
top-level runtime launch
temporary migration seams only
```

## Editor API owns

```text
future editor contracts
placeholder descriptors
inspector schema descriptors
provider traits
```

## Editor session owns

```text
placeholder editor session state
selection/document placeholder state
future editor coordination contracts
```

## Mods own

```text
authoring YAML
scene-local assets
mod-level reusable assets
scripts
content-specific values
```

Editor must not duplicate engine validation. It should consume engine metadata and diagnostics.

---

# Code Smells to Avoid

Avoid these patterns:

```text
app-level glue knowing specific domain internals
apps/app adding direct dependencies to 2d/3d/audio/ui domains
runtime/bundles defining App* types
renderer core containing feature-specific if/else branches
new v2 systems next to old systems
compatibility paths left after migration
huge functions with unrelated responsibilities
DTO duplication across engine/app/editor
YAML shape that cannot map to editor targets
runtime feature with no diagnostics
per-frame allocations in hot paths
thread spawn/join inside frame loop
debug overlay affected by game post-fx
engine contracts hidden inside app crate
manual parsing in editor that differs from engine parser
stringly typed target refs without validation
large public functions used as alternate pipelines
```

Specific architecture smells:

```text
DevConsoleRuntimePlugin owned by apps/app
debug_overlay implementation owned by apps/app
App*RenderExtractor inside runtime/bundles
AppFrameCompositionBuilder inside runtime/bundles
editor-api depends on UI frameworks
editor-session depends on winit/wgpu/apps/app
domains registered manually in bootstrap
```

---

# Change Granularity Rules

A good patch changes one conceptual layer.

Good sequence:

```text
1. data/contract model
2. registration helper
3. runtime use
4. diagnostics/tests
5. cleanup stale names/docs
```

Bad patch:

```text
adds editor API
adds renderer bridge
renames bundles
moves devtools
changes mod content
updates bootstrap
removes legacy
all at once
```

Keep phases small enough that each can be checked independently.

---

# Feature Implementation Shape

A complete runtime feature should normally include:

```text
1. Authoring/runtime model
2. Domain-owned capability registration
3. Validation/certification if relevant
4. Diagnostics/console visibility if runtime behavior is opaque
5. Tests
6. Editor-readable metadata placeholder if relevant
7. Cleanup of replaced paths
```

A placeholder editor capability should normally include only:

```text
stable id
component type id
minimal inspector schema
simple property descriptors
no UI
no editor workflow
```

---

# Diagnostics-First Rule

Do not add hard-to-debug runtime behavior without diagnostics.

Examples:

```text
new scheduler behavior -> scheduler.stats / scheduler.overrides
new post-fx -> postfx.cert / postfx.stats
new render path -> render.plan / render.graph
new particle optimization -> particles.stats / debug.particles
new input feature -> debug.input
new bundle registration -> architecture test or diagnostic visibility
```

Diagnostics may be simple, but must exist before the feature becomes opaque.

---

# Performance Rules

Avoid:

```text
per-frame allocation in hot paths
thread spawn per frame
immediate worker spawn + join pretending to be async
cloning large buffers per particle
string clones per draw command
offscreen render target when no post-fx needs it
rebuilding GPU pipelines per frame
reading world data from worker without snapshot/command model
```

Prefer:

```text
persistent worker pools
double buffers for async visual data
frame-local transient resource allocator
prepared render batches
cached pipelines
small immutable snapshots
command/result buffers
diagnostic counters
```

Render-specific:

```text
No post-fx:
  render world directly to surface.

With post-fx:
  render world to transient texture.
  post-fx samples texture.
  game UI after post-fx.
  debug overlay last.
```

---

# Scene / Mod Authoring Rules

Use scope-based authoring.

```text
mod-level folder = reusable for the whole mod
scene-level folder = local to that scene
```

Good:

```text
mods/rotten-club/ui/themes/rotten-noir.yml
mods/rotten-club/scenes/main-menu/ui/bindings.yml
mods/rotten-club/scenes/main-menu/visual/lens.yml
```

Avoid generic `parts/`.

Use domain folders:

```text
visual/
entities/
ui/
input/
events/
state/
audio/
timelines/
scripts/
```

Scene manifest should compose domains:

```yaml
use:
  visual:
    - ./visual/render.yml
    - ./visual/lens.yml
  ui:
    - ./ui/mount.yml
```

---

# Scheduler / Jobs Rules

Workers must not mutate world directly.

Good pattern:

```text
main thread:
  owns world mutation
  collects input
  applies command/results
  submits GPU work

workers:
  receive snapshots
  compute results
  return typed output
```

Avoid:

```text
spawn thread and immediately join every frame
worker directly locking live render state while renderer waits
jobs without stats
scheduler config that silently fails to match targets
```

Required diagnostics:

```text
scheduler.stats
scheduler.overrides
worker_jobs_submitted
worker_jobs_completed
worker_waited_this_frame
job_in_flight
reused_previous_frame
```

---

# Dev Console / Debug Overlay Rules

Console/debug behavior belongs in `engine/devtools` unless it is explicitly mod-specific.

Each public command should usually live in its own focused module.

Command descriptors must be accurate because completion uses them.

Use categories:

```text
debug
render
postfx
scheduler
particles
audio
input
scene
camera
editor
```

Completion/hinting must use registry descriptors, not hardcoded command lists.

Debug overlay must render after all game effects.

---

# Module and File Granularity

Prefer cohesive files.

Good:

```text
runtime/bundles/src/wgpu_render_extractors/composition.rs
runtime/bundles/src/wgpu_render_extractors/sprite2d.rs
engine/render-api/src/composition.rs
engine/camera/src/lib.rs
engine/devtools/src/console.rs
engine/devtools/src/debug_overlay.rs
engine/editor-api/src/inspector.rs
engine/editor-session/src/lib.rs
```

Bad:

```text
one 2000-line render.rs owning graph, resources, post-fx, UI, debug, and diagnostics
misc.rs
utils.rs with domain logic
manager.rs without clear responsibility
apps/app owning reusable engine contracts
```

Create a new file when:

```text
the concept has a stable name
it will be reused
it has tests or diagnostics
it reduces a monolithic file
it clarifies an architecture boundary
```

Do not create a new file for one tiny helper unless it clarifies a boundary.

---

# Naming Rules

Avoid:

```text
new
old
v2
legacy
manager
stuff
helper
misc
temp
experimental
App* for non-app types
```

Use domain language:

```text
FrameCompositionPlan
CompositionLayer
RenderSpace
WgpuFrameCompositionBuilder
WgpuSprite2dRenderExtractorBridge
CameraBinding
CameraService
InspectorSchema
PropertyDescriptor
EditorCapabilityDescriptor
DebugOverlayService
ConsoleCommandRegistry
```

Temporary migration names must be removed during cleanup.

---

# Preferred Task Templates

## Template: Close Bootstrap Seam

```text
1. change-plan/open-set for bootstrap + runtime bundles
2. trace current app-owned registrations
3. slice register_app_host_platform_plugins or equivalent symbol
4. identify registrations that belong in runtime/bundles or devtools
5. move behind existing helper or add focused helper
6. verify touched crates
7. update architecture tests/docs if boundary changed
```

## Template: Close Render Composition Seam

```text
1. trace FrameCompositionPlan / CompositionLayer / RenderSpace
2. slice WgpuFrameCompositionBuilder
3. ensure layers are explicit and sorted
4. ensure debug overlay is last
5. ensure no App* render bridge names remain
6. run targeted checks/tests
```

## Template: Add Domain Runtime Capability

```text
1. trace existing capability registration for that domain
2. slice owner registration symbol
3. add capability in domain or bundle helper, not apps/app
4. expose through runtime/bundles if it belongs in a bundle
5. verify domain crate and affected bundle/app crate
```

## Template: Add Editor Placeholder Capability

```text
1. trace existing editor_capability.rs in similar domain
2. slice minimal descriptor function
3. add stable id and component_type
4. add minimal InspectorSchema fields
5. do not add UI/framework/workflow
6. verify domain crate and editor-api if touched
```

## Template: Remove Legacy Name

```text
1. trace legacy name exactly
2. verify no required call-sites remain
3. replace with final architecture name
4. update docs/tests if they mention the stale name
5. trace again
6. check touched crates
```

## Template: Fix Compile Error

```text
1. run targeted cargo check through fallout
2. trace exact missing symbol/type
3. slice owner symbol
4. patch exact issue
5. rerun same check
```

---

# Common Codemap Recipes

## Find runtime bundle registration

```powershell
& $cm trace "register_full_runtime_capabilities" --limit 20
& $cm trace "RuntimeBundle" --limit 20
& $cm open-set "runtime bundles capability registration" --why --limit 12
```

## Work on WGPU render extraction bridges

```powershell
& $cm trace "WgpuFrameCompositionBuilder" --limit 20
& $cm trace "WgpuSprite2dRenderExtractorBridge" --limit 20
& $cm trace "WgpuMesh3dRenderExtractorBridge" --limit 20
& $cm open-set "WGPU render extractor bridges runtime bundles" --why --limit 12
```

## Work on render composition contracts

```powershell
& $cm trace "FrameCompositionPlan" --limit 20
& $cm trace "CompositionLayer" --limit 20
& $cm trace "RenderSpace" --limit 20
& $cm open-set "render-api composition layers render spaces" --why --limit 12
```

## Work on camera contracts

```powershell
& $cm trace "CameraBinding" --limit 20
& $cm trace "camera_by_binding" --limit 20
& $cm open-set "engine camera binding camera service" --why --limit 12
```

## Work on editor API placeholder

```powershell
& $cm trace "InspectorSchema" --limit 20
& $cm trace "PropertyDescriptor" --limit 20
& $cm trace "ComponentTypeId" --limit 20
& $cm open-set "editor-api placeholder inspector schema property descriptors" --why --limit 12
```

## Work on editor capabilities in domains

```powershell
& $cm trace "editor_capability" --limit 20
& $cm trace "InspectorSchema::placeholder" --limit 20
& $cm open-set "domain editor_capability minimal inspector schema" --why --limit 12
```

## Work on devtools

```powershell
& $cm trace "ConsoleCommandRegistry" --limit 20
& $cm trace "DebugOverlay" --limit 20
& $cm trace "DevtoolsRuntimeBundle" --limit 20
& $cm open-set "engine devtools console debug overlay diagnostics commands" --why --limit 12
```

## Check stale App names

Prefer codemap trace first:

```powershell
& $cm trace "AppFrameCompositionBuilder" --limit 20
& $cm trace "AppRenderExtractor" --limit 20
```

If needed, use a targeted search only in docs or a known directory:

```powershell
rg "AppFrameCompositionBuilder|AppRenderExtractor" docs AGENTS.md crates/apps/app/README.md
```

Do not start with repo-wide `rg`.

---

# Anchor Policy

Use codemap anchors for important architecture points.

Add/update anchors when touching:

```text
runtime bundle registration
frame composition builder
render-api composition contracts
camera service/binding
devtools registry
editor-api placeholder contracts
editor-session placeholder boundary
app bootstrap seam
app render_runtime seam
```

After anchor changes:

```powershell
& $cm anchors --write
& $cm anchor-check
```

Do not tag every file. Anchor important navigation points only.

---

# Final Response Checklist

Before reporting done:

```text
Did I use codemap-first?
Did I avoid broad scans?
Did I edit only targeted files?
Did I avoid adding domains to apps/app?
Did I avoid building apps/editor?
Did I keep editor-api/editor-session UI-free?
Did I avoid old raw ops --raw workflow?
Did I run ops-raw-check --yaml if raw ops were used?
Did I run verify-plan --changed?
Did I run only touched crate checks/tests?
Did I remove stale App* names introduced or affected by the task?
Did I avoid claiming unrun verification?
```

Final response format:

```text
Changed:
- file/symbol
- file/symbol

Verified:
- command
- command

Notes:
- skipped checks and why
- remaining follow-up
```

---

# Stop Conditions

Stop and report when:

```text
requested phase is complete
a required symbol cannot be found after trace/open-set
a change would require broad unrelated refactor
checks reveal unrelated pre-existing errors
task scope would expand beyond pasted plan
a requested change would violate app/domain/editor boundaries
```

Do not broaden the task without instruction.

---

# Summary

Use codemap to spend precision instead of tokens.

For Amigo refactors, do not solve domain-owned runtime logic by labeling it `app.host`.

The correct Amigo workflow is:

```text
targeted discovery
small patch
current raw ops protocol if needed
minimal verify
cleanup stale names
concise report
```

The correct Amigo architecture direction is:

```text
apps/app as thin host
domains own runtime capabilities
runtime/bundles composes capabilities
engine/devtools owns debug/dev runtime behavior
engine/render-api owns composition contracts
engine/camera owns camera contracts
editor-api/editor-session remain placeholders
no v2 systems
no permanent legacy
diagnostics-first
clean final names
```

```
```
