# Rotten Club Timeline

The main menu intro is intentionally small and data-driven. `scene.rhai` loads a
tiny lifecycle director, while all timing and animation tracks live in
`scenes/main-menu/timeline/intro.yml`.

## Beat Sheet

| Time | Beat | Runtime effect |
| ---: | --- | --- |
| 0.00s | Reset | Hide layered image lights, title text, beacon lights, and rain emitters. |
| 2.40s | Skyline | Fade in the distant skyline layer. |
| 5.00s | Far street | Raise base image opacity so the alley reads before neon signs appear. |
| 5.60s | Bar | Fade amber bar sign and lantern layers. |
| 7.20s | Club | Fade club sign and club entry layers; start subtle beacon flicker. |
| 9.00s | Pharmacy | Fade the green pharmacy cross layer. |
| 10.00s | Club focus | Let club sign/beacons become the main visual anchor. |
| 11.00s | Rain | Ramp two particle emitters and rain relief edges. |
| 14.20s | Lightning | Fire a short flash that boosts rain and beacon intensity. |
| 15.00s | Title focus | Prepare the quiet title reveal after the lightning beat. |
| 17.00s | Title | Show title/subtitle and ramp title alpha. |
| 19.00s | Complete | Mark `intro.complete` and stop one-shot intro logging. |

## Ownership

- `timeline/intro.yml` owns layer opacity, beacon intensity, rain intensity,
  camera focus, title alpha, and lightning state through `RuntimeControlService`
  paths.
- `lighting/groups.yml` owns declared light groups, camera response, and light routes.
- `state/defaults.yml` owns deterministic reset values.
- `scripts/packages/main_menu/director.rhai` owns lifecycle/debug logging only.

The director does not create renderer-specific behavior. The timeline writes
plugin-owned control paths and state keys, and the renderer consumes the declared
scene contracts.
