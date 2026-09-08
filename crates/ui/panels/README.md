# Runtime panels

Engine-owned panel lifecycle, metadata validation, child-process transport,
HotReloadService watches and domain-owned presets. Registered by runtime bundles;
scenes without panel declarations remain unaffected. Rhai interacts through the
same typed controls and event queue as other consumers. Test with
`cargo test -p amigo-panels`.
