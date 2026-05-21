# Diagnostics map

## Camera optics

```text
camera.optical.contributions
camera.optical.candidates
camera.optical.targets
```

## Render composition

```text
render composition diagnostics
frame graph node status
visual source buffer status
post-fx execution/debug views
```

## Scene

```text
hydration warnings
component metadata validation
scene command validation
```

## Plugin

```text
plugin capabilities
plugin waterfall tests
plugin diagnostics channels from plugin.toml
```

Rule: new explicit contracts should have diagnostics showing when they are absent, consumed, or skipped.
