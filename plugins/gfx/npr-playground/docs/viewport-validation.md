# Windows viewport validation — 2026-09-12

## Drawing Studio document-format checkpoint

The unified authored layer list and explicit migration are implemented. Versioned
brush resources, per-instance targets, procedural masks/groups, the Brush Editor,
build-up and named variants are implemented and covered by owner-crate tests.
The cache still keys packets by effective style/layers; zero path rebuilds on
colour/opacity edits are covered by the owner-crate cache test, and the full
4/8/16-layer performance matrix is recorded below. Performance acceptance still
depends on the p95 and memory criteria.

Real-window QA ran before changes on the existing optimized executable and again
after `cargo build --profile playground -p amigo-app`. Both runs passed Gallery
single-window lifecycle, Cube without companion, bounded-queue recovery, DPI 1/2,
resize, mode switching, form focus, device-loss recovery and engine exit. In the
new build Native GPU and Local RGBA matched exactly at 640×360; JPEG mean absolute
channel error was `[0.866, 0.738, 0.877]` on the 0–255 scale. Final images/report
are under `target/viewport-qa`; the initial capture is retained under
`target/drawing-studio-baseline-qa`.

These captures do not freeze the same sketch epoch across launches. They verify
transport agreement, not deterministic migration image equivalence or p95
regression. The current build now has a resumed 4/8/16-layer matrix and a
five-minute resource test recorded below. Migration tests verify exact layer
parameters, order, roundtrip, object patches, backup bytes, duplicate rejection
and external-file conflicts.

The machine-readable acceptance summary is generated with `python plugins/gfx/npr-playground/tools/final_acceptance.py` and written to `target/drawing-studio-acceptance.json`. The diagnostic rerun is recorded in `target/drawing-studio-acceptance-diagnostics.json`: all 648 matrix combinations are valid, 8,553 diagnostic samples report 19,793 stale-readback drops, and the 300-second soak ends with `engine_exited: true`. It reports `performance_signoff: false`; the current worst p95 is 67.3 ms for multi/pencil-study, 8 layers, JPEG, orbit.

The viewport worker uses asynchronous readback with latest-frame-wins: when
multiple readbacks complete together, superseded permits are released and only
the newest frame is sent to the presentation bridge.
The per-frame header now carries `stale_readback_drops`; a fresh smoke run
confirmed the field is present and reports zero drops for the isolated cube
case.

## Previous viewport measurements

Lifecycle update: Gallery now displays only Tauri and exits its windowless engine
when Tauri closes. Cube still displays one Winit window. The 216-case performance
matrix and Winit comparisons below predate this change and describe the earlier
two-window setup; they are retained as historical measurements. The real-window
QA script now checks window count and engine exit for the single-window setup.
That QA passed: Gallery showed exactly one Tauri window, Cube showed exactly one
Winit window, and closing Tauri ended its engine process. The five lifecycle
tests and three transport tests also passed with the new host policy.
An additional nine-case cube smoke run covered orbit, spin and pause in all
three modes and confirmed engine exit on close (`target/viewport-single-window.jsonl`).
It was functional validation with existing user applications still running,
not a replacement performance baseline.

Native GPU, Local RGBA and JPEG are connected to the real NPR scene and Tauri
companion. Functional validation passed on the machine below. The performance
acceptance is **not fully met**: simple models approach 60 FPS, but presentation
interval p95 exceeds 20 ms in some cases. Suzanne also remains extraction-bound.

## Reproduce

From the repository root, after installing the frontend dependencies:

```powershell
rtk cargo build --profile playground -p amigo-app
rtk proxy python -m pip install -r plugins/gfx/npr-playground/tools/requirements.txt
rtk proxy python plugins/gfx/npr-playground/tools/viewport_qa.py
rtk proxy python plugins/gfx/npr-playground/tools/viewport_benchmark.py --seconds 2 --soak 300 --output target/viewport-validated.jsonl
rtk proxy python plugins/gfx/npr-playground/tools/viewport_benchmark.py --skip-matrix --soak 300 --output target/viewport-resource-validation.jsonl
```

The benchmark accepts `--layers 4,8,16` (the default) and records the selected
stack size on every case. The current full matrix covers all three stack sizes;
the raw output and aggregate table below are from that run.

The scripts launch and close only their own processes, use an isolated WebView2
profile and an ephemeral localhost DevTools port, and never save authored edits.
Visual capture uses the owned native child window or canvas, not the desktop.
Logs, raw samples and QA images are written under `target` and are not source
artifacts. The resource-only run disables retained per-frame measurement data.

## Method

- Windows; Intel Core i5-12600K; NVIDIA GeForce RTX 3070 Ti.
- Optimized `playground` build with debug symbols. No concurrent builds during
  measurements. Normal Winit renderer: Vulkan; Native GPU worker: DX12.
- 216 cases: cube, sphere, Suzanne and a three-object scene; `comic-ink` and
  `pencil-study`; 640×360, 960×540 and 1280×720; three transports; orbit, spin, pause.
- Each case has 1.2 seconds to settle and a two-second measurement window.
  Adaptive resolution is off. Models use the gallery camera (distance 14);
  camera angles and spinning poses progress between cases.
- FPS and interval p95 come from actual Tauri presentation callbacks. Input p95
  includes the oldest queued controller movement through its acknowledged frame.
  It excludes OS delivery before the controller and physical monitor scanout.
- Extraction, CPU submit, asynchronous readback and encoding are measured
  separately. Submit is CPU preparation/submission time, not a GPU timestamp.
  Short samples describe this run, not a statistical guarantee across machines.

The previous [offscreen analysis](viewport-performance-analysis.md) used different
camera/framing and timing boundaries. Its dev/release numbers are historical
context, not a controlled before/after speedup claim. The complete matrix was
followed by generation-scoped error and rapid-resize hardening; final QA and the
resource-only run exercise those changes.

## Measured results

All 216 cases produced valid measurements with no rejected mutations or reported
presentation errors. All 72 pause cases produced zero continuous frames after
settling. Simple models delivered 58.7–59.7 FPS orbit and 29.9–30.1 FPS spinning
across the matrix. Their worst input p95 was 35.5 ms; worst interval p95 was
26.9 ms. Fresh-process startup to the first Native GPU HUD took 2.01 seconds
with a fresh client profile and warm OS caches.

Orbit at 1280×720 (N/R/J = Native GPU / Local RGBA / JPEG):

| Model / look | FPS N / R / J | Native extraction mean (ms) | Native interval / input p95 (ms) |
|---|---:|---:|---:|
| Cube / comic-ink | 59.6 / 59.2 / 59.3 | 0.08 | 21.2 / 25.2 |
| Cube / pencil-study | 59.5 / 59.4 / 59.6 | 0.26 | 20.6 / 25.6 |
| Sphere / comic-ink | 59.4 / 59.4 / 59.1 | 0.10 | 20.2 / 24.9 |
| Sphere / pencil-study | 59.5 / 59.6 / 58.9 | 0.35 | 20.6 / 21.2 |
| Suzanne / comic-ink | 56.8 / 48.3 / 46.7 | 5.53 | 27.1 / 41.4 |
| Suzanne / pencil-study | 53.3 / 34.3 / 33.9 | 11.79 | 30.1 / 52.1 |
| Three objects / comic-ink | 53.8 / 47.7 / 45.9 | 5.91 | 28.2 / 40.9 |
| Three objects / pencil-study | 50.7 / 36.4 / 39.3 | 12.33 | 28.3 / 55.5 |

Suzanne/pencil Local RGBA reached 55.2 ms interval p95 and 83.7 ms input p95;
JPEG reached 55.6 ms and 90.3 ms. Removing JPEG alone does not resolve extraction
cost. Native GPU reported zero readback and encoding time in every sampled frame;
its producer path uses shared textures/fences and GPU copies. Cube/comic JPEG
at 1280×720 averaged 3.66 ms readback and 2.46 ms encoding at quality 92.

The first five-minute switching/resize run kept total process handles between
4,453 and 4,470, with no presentation errors. Private memory grew from 1,467.6 to
1,528.6 MiB, with most of the increase in the WebView renderer. This run also
retained the benchmark's frame history. The engine stayed within 730.2–735.7 MiB
and the companion within 100.2–102.2 MiB.

The separate final-build resource run completed 300 seconds and 30 mode/resize
cycles without errors or automatic companion restart. After the first cycle
initialized all transports, handles varied between 4,478 and 4,549 without a
steady increase. Private memory varied between 1,373.6 and 1,465.0 MiB; the final
sample was the maximum. WebView memory fell during the run (total 1,450.6 →
1,386.9 MiB), but disabling retained measurement samples did not eliminate the
higher final footprint. Engine private memory changed from 704.2 to 713.1 MiB
and companion memory from 98.2 to 101.6 MiB between the first completed cycle and
the end. **The zero-growth memory criterion is not established by these runs.**

The layer-count matrix completed on the current build with 648/648 unique valid cases
and zero rejected mutations. It covered cube, sphere, Suzanne and the
multi-object scene; both looks; all three sizes, transports and scenarios; and
stacks of 4, 8 and 16 layers. Aggregated maxima/minima were:

| Layers / transport | Interval p95 max (ms) | FPS min | Extraction mean (ms) |
|---|---:|---:|---:|
| 4 / Native GPU | 38.3 | 29.80 | 4.160 |
| 8 / Native GPU | 38.5 | 29.87 | 4.330 |
| 16 / Native GPU | 39.6 | 29.85 | 4.271 |
| 4 / Local RGBA | 62.6 | 29.71 | 4.347 |
| 8 / Local RGBA | 60.0 | 29.56 | 4.292 |
| 16 / Local RGBA | 65.3 | 29.36 | 4.271 |
| 4 / JPEG | 56.3 | 29.62 | 4.374 |
| 8 / JPEG | 55.4 | 29.61 | 4.336 |
| 16 / JPEG | 61.6 | 29.65 | 4.365 |

The full raw result is `target/viewport-layer-matrix.jsonl` from the baseline run;
the resumed brush-default run is `target/viewport-layer-matrix-brush-defaults-full.jsonl`.
Both are generated under `target`, so they are not source artifacts.

After enabling pinned brush-definition defaults, a focused cube/comic-ink Native
GPU run (640×360, 4/8/16 layers) produced 9/9 valid cases. Orbit extraction
means were 0.075, 0.071 and 0.089 ms; spin means were 0.081, 0.078 and 0.086 ms.
The corresponding interval p95 values stayed between 21.0 and 37.7 ms. Raw
results are in `target/viewport-brush-defaults-cube.jsonl`.

The current-build resource soak completed the requested 300 seconds and ended
with `engine_exited: true`. Across 30 samples, the host/engine handle counts
remained bounded (403–525, including process transitions), while WebView
processes varied with transport (149–1379 handles). Private memory varied from
103 MiB to 432 MiB for host/engine processes and from 2.4 MiB to 421 MiB for
WebView processes, so the zero-growth plateau is still not established even
though the application closed cleanly.

The previously unstable Suzanne/JPEG case was rerun in isolation after the
session and harness changes. Suzanne at 640×360, JPEG and 16 layers completed
orbit, spin and pause successfully (3/3 valid); orbit extraction averaged
6.48 ms with 41.8 ms interval p95. This isolates the earlier WebSocket abort to
the long multi-case switching run rather than the individual render case.

The Winit host frame loop averaged 401.0 iterations/s with the animated companion
and 437.5 after closing it (eight/nine one-second samples). This measures the host
RenderExtract cadence, not monitor presentation FPS. The host remained running
and did not reopen the manually closed companion during the next ten seconds.

## Functional evidence

Final real-window QA verified Native GPU as the first-run default, restoration of
the last successful JPEG preference, JPEG → DX12 → Local RGBA switching, preserved
selection, overlapping resize/switch requests, emulated DPR 2 and return to DPR 1
in all modes, modal occlusion, and form focus excluding camera shortcuts.
Injected WebGL loss preserved the previous JPEG view on a failed switch; losing
active Local RGBA allowed a manual Native GPU choice. Cube launched no companion.

The transport-overload regression reproduced `sending on a full channel` and
Tauri termination on the previous executable. After retaining pending input
under backpressure, the same real-window test survived 256 viewport requests
around a temporary engine pause and successfully switched presentation again.
The queue remains bounded; a socket-level test additionally verifies ordered
delivery of 128 actions, replies during saturation and shutdown while full.

At 640×360 on the same paused cube/pencil scene, Native GPU and Local RGBA had
identical pixels, orientation and dimensions. JPEG mean absolute channel errors
were R 0.866, G 0.738, B 0.877 on the 0–255 scale. A separate Q92 codec test compares
libjpeg-turbo with the previous JPEG encoder using color blocks and ink edges.

Validation completed during implementation:

| Owner / check | Result |
|---|---|
| `cargo test -p amigo-playground-api` | 6 passed |
| `cargo test -p amigo-render-npr` | 100 passed |
| `cargo test -p amigo-npr-playground-plugin` | 71 passed, including layer cache, brush versions and migration |
| `cargo test -p amigo-playground-native` | 4 passed, including authentication rejection |
| `cargo test -p amigo-render-wgpu shared_texture_roundtrip -- --ignored` | Passed on DX12: three sizes, slot reuse and GPU retirement |
| `cargo test -p amigo-runtime-bundles --test playground_jpeg` | 1 passed |
| `cargo test -p amigo-runtime-bundles playground_ --lib` | 3 passed, including transport backpressure |
| `amigo-plugin-check validate plugins/gfx/npr-playground` | Passed |
| App `scene_loading_tests::threed::npr_` / `--test playground_lifecycle` | 3 / 5 passed |
| Checks: render-wgpu, runtime-bundles, playground-tauri, app | Passed |
| Frontend `npm run check`, `npm test`, `npm run build` | 0 errors, 3 DOM-reference warnings, 10 tests passed, build passed |
| `cargo build --profile playground -p amigo-app` | Passed, actual executable used by QA |
| `git diff --check` | Passed |

Unit coverage includes accumulated small movements/wheel deltas, sticky drag
threshold, one undo entry per gesture, request-ID/revision conflicts, stale
generations/ACKs, two-frame credit limits, sketch pause/epoch rebuild counts and
shared prepared surfaces. Real GPU tests require a Windows desktop and are
explicitly invoked rather than silently skipped as evidence.

## Remaining acceptance work

- Reduce frame pacing jitter to reach the 20 ms interval-p95 target reliably;
  continue optimizing Suzanne extraction and transfer overlap. The completed
  layer matrix identifies the affected model/transport combinations above.
- Investigate the WebView/GPU memory footprint over longer steady workloads and
  establish a stable plateau after initialization; the five-minute runs show
  stable handle counts but a higher final private-memory footprint.
- Validate physical monitor DPI transitions and actual DX12 device removal;
  this run used emulated DPR and injected WebGL loss, not a driver reset.
- Repeat longer samples on additional adapters and machines. These five-minute
  tests cannot prove absence of every resource leak.
- macOS/Linux Native GPU and Local RGBA ports remain unimplemented. JPEG is the
  portable transport; no automatic mode fallback is introduced.
