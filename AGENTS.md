# AGENTS.md

This file defines how coding agents must work in the Amigo repository.

Amigo is a fresh Rust engine project. Do not preserve legacy paths after a migration is complete. Prefer clean final names over compatibility shims, `v2` modules, duplicate paths, or fallback behavior that hides architecture problems.

## Prime directive

Use targeted navigation first. Do not scan the repository blindly.

Preferred discovery order:

```powershell
git status --short
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"
& $cm brief
& $cm changes --compact --hide-generated --limit 20
& $cm change-plan "<task>" --limit 20
& $cm open-set "<symbols / paths / topic>" --why --limit 20
```

`amigo-codemap` is the primary repository-aware navigation tool. Use it first for task scope, architecture boundaries, changed-file context, verification planning, move/rename planning, and Amigo-specific guardrails.

Use `codegraph` as a secondary symbol/callgraph tool when it is initialized and the question is about precise code structure, for example:

```text
where is this symbol defined?
what calls this function?
what does this function call?
what source implements this narrow flow?
what is the likely blast radius of this symbol-level change?
```

Do not use `codegraph` to replace `amigo-codemap` planning. Do not initialize or rebuild a large external index by default; only do so when the task clearly benefits from symbol/callgraph precision and the user-facing cost is lower than manual exploration.

If `amigo-codemap` is unavailable, use narrowly scoped `rg` commands and report that fallback explicitly. Never start with broad file-by-file browsing.

If `codegraph` is unavailable or not initialized, continue with `amigo-codemap` plus narrow `rg` and report that limitation only when it affects confidence.

Do not open generated concat snapshots during normal repository work. A concat snapshot is only an external review artifact, not a source file.

## Current architecture

```text
apps/app                 = thin runtime host and bootstrap only
crates/runtime/bundles   = runtime composition and backend bridges
crates/engine/runtime    = plugin/system runtime contracts
crates/engine/session    = frame/session scheduling services
crates/engine/scene      = scene documents, hydration, metadata, commands
crates/engine/render-api = renderer-facing contracts and frame graph models
crates/engine/render-wgpu= WGPU backend implementation
crates/engine/camera     = shared camera contracts/services
crates/engine/devtools   = dev console, debug overlay, diagnostics commands
crates/engine/editor-api = editor contracts only
crates/engine/editor-session = editor session contracts only
crates/engine/editor-authoring = authoring graph/cache services
crates/engine/editor-ingame   = runtime in-game editor overlay/mockup
plugins/<family>/<plugin>     = domain-owned plugin implementations
mods/                         = content, scenes, scripts, authored data
```

No standalone `apps/editor` should be created unless a task explicitly changes the editor strategy.

## General project direction

Amigo is still evolving quickly. Prefer clean final boundaries over preserving intermediate migration shapes.

Default direction for architectural work:

```text
authored data / scene model
  -> domain-owned services and extractors
  -> neutral engine contracts
  -> backend execution
  -> diagnostics
```

Core rule:

```text
plugins/domains describe intent
engine contracts describe shared data and boundaries
backends execute contracts
apps/app hosts and composes
```

The renderer should execute declared contracts. It should not guess domain intent from object existence, names, debug strings, or accidental side effects.

## Hard prohibitions

Do not add by default:

```text
legacy_* modules
v2 modules
parallel replacement systems
compatibility shims
fallbacks that silently hide missing contributions
renderer-side domain guessing
app-side domain wiring
new dependencies in apps/app for domain behavior
standalone apps/editor
workspace-wide rewrites
large formatting-only diffs
```

Do not run by default:

```text
cargo check --workspace
cargo test --workspace
full repo formatting
broad find/cat/Get-Content scans
```

Use workspace-wide commands only when the task explicitly requires them or targeted validation cannot prove safety.

## Operation protocol

Every implementation instruction should be expressed as one of these operations:

```text
READ    inspect a precise file, symbol, or small range
ADD     create a new file or symbol
MODIFY  change an existing file or symbol
DELETE  remove obsolete code after replacement is complete
MOVE    move/rename code without behavior change
```

For each operation include:

```text
- exact path
- exact symbol or line range when available
- intent
- patch or code snippet when practical
- validation command
- what not to change
```

Prefer small manual patches over broad generated rewrites.

## File reading rules

Use this order:

1. `amigo-codemap change-plan` for task scope.
2. `amigo-codemap open-set --why` for candidate files.
3. `codegraph_explore`, `codegraph_search`, `codegraph_callers`, or `codegraph_callees` only for initialized symbol/callgraph questions where it avoids multiple file reads.
4. `amigo-codemap symbols` or `rg -n "SymbolName" <narrow-path>`.
5. `amigo-codemap slice` or a narrow line range.
6. Full file read only if the file is small or the symbol context is insufficient.

Choose the tool by job:

```text
amigo-codemap = repository scope, architecture, changed files, worksets, verify plans, move plans, guardrails
codegraph     = initialized symbol definitions, callers/callees, narrow source bodies, symbol-level impact
rg            = narrow fallback or literal text check after the scope is known
```

Avoid opening large files such as renderer init, metadata registries, generated docs, or snapshots unless the task requires a specific range.

## Cost control rules

Optimize for the smallest practical amount of:

```text
LLM tokens
file reads
command output
crate rebuilds
cross-crate verification
architectural churn
```

Use this execution order by default:

```text
1. locate exact symbols
2. read only the minimum context
3. patch one conceptual seam
4. run one cheapest relevant check
5. escalate verification only if the previous step passes
```

Prefer:

```text
trace / narrow rg over broad search
small range reads over full-file reads
owner-crate cargo check over downstream checks
targeted test filters over full package tests
one precise follow-up patch over speculative rewrites
```

Avoid:

```text
running app-level checks before lower layers are green
rerunning the same broad command before reading the first failure
mixing live-path and test/support-path searches in one wide query
open-set for a seam that is already well understood
checking multiple downstream crates before the owner crate passes
```

When searching for "remaining references", separate results into:

```text
live runtime/backend path
tests/support path
diagnostics/stats path
```

Do not treat those as one problem class.

If the next step would be very expensive in tokens, file reads, rebuild time, generated output, or broad validation, stop before doing it and say:

```text
hej stop mam pomsysl jak ulepszyc narzedzie zeby zuzywac jeszcze mniej tokenow
```

Then propose the specific tool or workflow improvement that would reduce cost, for example a new `amigo-codemap` query, narrower `codegraph` lookup, targeted invariant check, smaller slice command, or missing index/metadata feature. Do not use this stop phrase for routine cheap commands.

## Layer rules

### apps/app

Allowed:

```text
host startup
window/platform bootstrap
runtime preset selection
top-level app lifecycle
render frame submission seam
```

Forbidden:

```text
domain extraction logic
scene component behavior
camera optical policy
lighting/material/VFX semantics
plugin-specific scripting bindings
```

### crates/runtime/bundles

Allowed:

```text
compose runtime bundles
register plugins
connect backend bridge modules
assemble default presets
adapt domain extractors into backend packets
```

Forbidden:

```text
owning domain semantics
duplicating domain extraction logic
becoming a second app layer
adding app-specific names to generic runtime paths
```

### crates/engine/render-api

Allowed:

```text
render contracts
frame graph models
camera capture inputs
render target/contribution/candidate models
post-fx document/model contracts
renderer-facing diagnostics models
```

Forbidden:

```text
WGPU implementation details
domain-specific execution heuristics
app bootstrap knowledge
```

### crates/engine/render-wgpu

Allowed:

```text
WGPU pipelines
backend resources
render passes
texture/buffer management
implementation of render-api contracts
```

Forbidden:

```text
authoring semantics
scene/domain policy decisions
new renderer guesses based on component names
silent fallbacks for missing plugin contributions
```

### crates/engine/scene

Allowed:

```text
scene document model
hydration
scene commands
component metadata provider contracts
validation
```

Forbidden:

```text
renderer execution policy
WGPU-specific fields
plugin-specific runtime systems when provider registration is available
```

### plugins

A plugin owns its domain waterfall:

```text
source/document
  -> roles / capabilities
  -> contribution
  -> response
  -> coverage
  -> candidate
  -> target
  -> consumer
  -> diagnostics
  -> tests
```

A plugin should not execute another plugin's effect directly. It should declare contributions, candidates, targets, and diagnostics through contracts.

### mods

Mods own authored content only:

```text
scenes
scripts
assets
profiles
routes
```

Mods must not define engine behavior through renderer hacks.

## PostFX and camera optics rules

PostFX is the current high-risk area.

Do not introduce:

```text
PostFx2dV2
new central renderer switches without an audit note
new hardcoded camera debug ordering without a descriptor plan
new effect-specific WgpuSceneRenderer fields without checking for registry migration
new visual-source heuristics based only on object existence
```

Preferred direction:

```text
PostFxRenderDescriptor
PostFxRenderInput
PostFxRenderOutput
PostFxDebugPolicy
PostFxCachedImagePolicy
WgpuPostFxPipelineRegistry
coverage/render adapter registry
```

Camera optics should flow through explicit contracts:

```text
CameraOpticalResponse2d
CameraOpticalCoverage2d
CameraOpticalCandidate2d
CameraOpticalRenderTargetPlan
SceneHighlight
SceneEmissive
```

Do not make bloom/lens artifacts depend on implicit guesses such as "a lightmap exists". Use declared contribution roles and camera response metadata.

## Documentation rules

Keep documentation short, canonical, and UTF-8 clean.

`AGENTS.md` is for agent behavior only. It must not contain pasted chat introductions, outer markdown fences, broken encoding, or old snapshot names.

`PROJECT.md`, if present, is the canonical project-state overview.

`README.md` is the human entrypoint.

`docs/architecture/**` contains architectural source-of-truth documents.

`arch.md` or pasted long plans are not canonical unless explicitly promoted and cleaned.

For plugin docs, avoid one-line placeholders when a plugin is touched. A touched plugin should have meaningful docs for:

```text
README.md
docs/pipeline.md
docs/contributions.md
docs/diagnostics.md
tests/waterfall_tests.rs
```

Do not update unrelated plugin docs just to make the tree look complete.

## Validation rules

Always start and end with:

```powershell
git status --short
```

For code changes, run the smallest relevant command:

```powershell
cargo check -p <crate>
cargo test -p <crate> <test-filter>
```

Default verification ladder:

```text
1. targeted rg invariant check
2. cargo check -p owner-crate
3. cargo check -p first downstream crate only if needed
4. cargo check -p app only if app files or shared request/packet types changed
5. cargo test only when behavior changed or a touched test suite is the real owner
```

Do not jump to a more expensive tier before the cheaper tier is green.

For docs-only changes:

```powershell
git diff --check
```

For architecture-sensitive changes, also run targeted search checks, for example:

```powershell
rg -n "legacy|v2|compat|fallback" crates plugins docs
rg -n "apps/app" crates/runtime crates/engine plugins
```

Use these checks narrowly and interpret results. Do not report raw logs unless needed.

When a command fails:

```text
1. read the first real error
2. inspect only the first failing file/symbol
3. patch that issue
4. rerun the same cheapest relevant command
5. only then broaden validation if needed
```

Do not respond to a local compile error by immediately running broader checks.

## Reporting format

Final reports must be concise and factual:

```text
Wykonane:
- MODIFY path/to/file.rs - short change
- ADD path/to/new_file.rs - short change

Weryfikacja:
- cargo check -p crate-name - pass/fail
- cargo test -p crate-name filter - pass/fail

Ryzyka / poza zakresem:
- item - reason

Następny krok:
- one concrete next action
```

Never call partial work complete. Do not say "should work" as validation. Report the first real error when validation fails.
