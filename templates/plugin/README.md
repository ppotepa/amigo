# amigo.family.plugin-name

This plugin owns one domain waterfall from authored/source intent through participation, targets, diagnostics and tests.

Before adding behavior, update `plugin.toml` capabilities/slots/targets to describe the real contract. Keep renderer/backend execution behind the declared render boundary and keep scripting hooks inside `src/scripting`.

Validation:

```sh
cargo run -p amigo-plugin-check -- validate --plugins plugins/family/plugin-name
```
