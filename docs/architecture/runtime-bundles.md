# Runtime Bundles

`crates/runtime/bundles` composes runtime plugins.

It may:

- register domain plugins,
- register backend bridges,
- assemble runtime presets.

It must not:

- own domain logic,
- become a second app,
- duplicate domain extractors,
- know domain internals beyond plugin composition.

The WGPU extractor module is a backend bridge. Domain extraction logic belongs in domain crates.
