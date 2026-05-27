use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::ServiceRegistry;

use crate::{
    ComponentGraphProviderRegistry, ComponentHydratorRegistry, ComponentMetadataProvider,
    ComponentMetadataProviderRegistry, ComponentSchemaRegistry, PluginComponentGraphProvider,
    PluginComponentHydrator, SceneComponentPayload, SceneComponentSchemaProvider,
    ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry,
};

pub trait SceneComponentPluginSpec {
    const PLUGIN_ID: &'static str;
    const COMPONENT_TYPE: &'static str;

    type Payload: SceneComponentPayload + 'static;
    type DescriptorProvider: ScenePluginDescriptorProvider + Default;
    type SchemaProvider: SceneComponentSchemaProvider + Default + 'static;
    type PluginHydrator: PluginComponentHydrator + Default + 'static;
    type GraphProvider: PluginComponentGraphProvider + Default + 'static;
    type MetadataProvider: ComponentMetadataProvider + Default + 'static;
}

pub fn register_scene_component_plugin_spec<P>(registry: &mut ServiceRegistry) -> AmigoResult<()>
where
    P: SceneComponentPluginSpec,
{
    let descriptor_provider = P::DescriptorProvider::default();
    let schema_provider = P::SchemaProvider::default();
    let plugin_hydrator = P::PluginHydrator::default();
    let graph_provider = P::GraphProvider::default();
    let metadata_provider = P::MetadataProvider::default();
    let mut descriptors = ScenePluginDescriptorRegistry::new();
    descriptors.register_provider(&descriptor_provider);
    validate_descriptor_set::<P>(&descriptors)?;

    validate_component_type(
        P::COMPONENT_TYPE,
        schema_provider.component_type(),
        "schema",
    )?;
    validate_component_type(
        P::COMPONENT_TYPE,
        plugin_hydrator.component_type(),
        "plugin hydrator",
    )?;
    validate_component_type(
        P::COMPONENT_TYPE,
        graph_provider.component_type(),
        "graph provider",
    )?;

    let metadata = registry.required::<ComponentMetadataProviderRegistry>()?;
    metadata.register(metadata_provider);

    let schemas = registry.required::<ComponentSchemaRegistry>()?;
    for descriptor in descriptors.iter() {
        schemas.try_register_descriptor(descriptor.clone())?;
    }
    schemas.register_schema_provider(schema_provider);

    let hydrators = registry.required::<ComponentHydratorRegistry>()?;
    hydrators.register_plugin(plugin_hydrator);

    let graph_providers = registry.required::<ComponentGraphProviderRegistry>()?;
    graph_providers.register(graph_provider);

    Ok(())
}

fn validate_descriptor_set<P>(descriptors: &ScenePluginDescriptorRegistry) -> AmigoResult<()>
where
    P: SceneComponentPluginSpec,
{
    if descriptors.len() != 1 {
        return Err(AmigoError::Message(format!(
            "plugin spec `{}` must register exactly one scene component descriptor, got {}",
            P::PLUGIN_ID,
            descriptors.len()
        )));
    }

    let Some(descriptor) = descriptors.iter().next() else {
        return Err(AmigoError::Message(format!(
            "plugin spec `{}` did not register a scene component descriptor",
            P::PLUGIN_ID
        )));
    };

    if descriptor.id.as_str() == P::COMPONENT_TYPE {
        return Ok(());
    }

    Err(AmigoError::Message(format!(
        "descriptor component type `{}` does not match plugin spec component type `{}`",
        descriptor.id.as_str(),
        P::COMPONENT_TYPE
    )))
}

fn validate_component_type(
    expected: &'static str,
    actual: &'static str,
    provider_kind: &'static str,
) -> AmigoResult<()> {
    if actual == expected {
        return Ok(());
    }

    Err(AmigoError::Message(format!(
        "{provider_kind} component type `{actual}` does not match plugin spec component type `{expected}`"
    )))
}
