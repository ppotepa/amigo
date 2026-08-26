# Amigo fuzz targets

Install cargo-fuzz and run from the repository root:

```sh
cargo install cargo-fuzz
cargo fuzz run plugin_manifest
cargo fuzz run scene_yaml
cargo fuzz run rhai_source
```

The targets exercise the public parsing boundaries without assuming successful input. Crashes, panics, allocator failures, or hangs are bugs; ordinary parse errors are expected.
