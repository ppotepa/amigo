# Component metadata ownership

`crates/engine/scene/src/component_metadata.rs` is the built-in compatibility catalog, not the extension point for new domain metadata.

New plugin/domain component descriptors must be supplied through `ComponentMetadataProvider` and registered in `ComponentMetadataProviderRegistry`. Consumers should build the effective catalog with `ComponentMetadataProviderRegistry::compose`, using built-in descriptors only as the base set.

This keeps metadata ownership next to the plugin that owns hydration, runtime behavior, scripting and diagnostics, and prevents the historical central metadata file from continuing to grow with every new domain.
