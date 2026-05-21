# Runtime phases

Use this as a conceptual map; confirm exact phase names in `crates/engine/runtime` before editing.

```text
bootstrap
plugin registration
asset/config load
scene load
hydration
command application
simulation/update
extraction
render submission
diagnostics/devtools
shutdown/hot reload
```

Phase ownership rule:

```text
runtime owns scheduling and registration;
plugins own domain behavior;
render-api owns contracts;
render-wgpu owns backend execution.
```
