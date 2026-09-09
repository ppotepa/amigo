# Runtime panels

Engine-owned panel lifecycle, metadata validation, child-process transport,
HotReloadService watches and domain-owned presets. Registered by runtime bundles;
scenes without panel declarations remain unaffected. Rhai interacts through the
same typed controls and event queue as other consumers. Test with
`cargo test -p amigo-panels`.

`PanelService::snapshots` and `PanelService::apply_interaction` form the
transport-independent host seam. External egui uses the framed IPC equivalent;
the embedded engine-overlay host renders `PanelSnapshot` and returns
`PanelInteraction` with its generation and revision rather than mutating runtime
controls directly. Its current interactive core covers buttons, toggles, sliders,
direct-selection option cards with authored vector previews, expandable
dropdowns, persistent tab selection, locally collapsible group boxes and a
clipped, mouse-wheel scroll viewport for long embedded content; richer widgets
remain transport-neutral follow-up work.
