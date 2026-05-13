# Runtime Bundles

`crates/runtime/bundles` composes runtime plugins and backend bridges.

It may:

- register domain plugins,
- register backend bridges,
- assemble runtime presets.

It must not:

- own domain logic,
- become a second app,
- duplicate domain extractors,
- know domain internals beyond plugin composition,
- reintroduce `App*RenderExtractor` names outside `apps/app`.

The WGPU render extractor module is a backend bridge. It may adapt domain
extractors into `WgpuRenderFramePacket`, but domain extraction logic must remain
in domain crates.
