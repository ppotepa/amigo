use std::sync::Arc;

use amigo_3d_material::MaterialPlugin;
use amigo_3d_mesh::{MeshPlugin, MeshSceneService};
use amigo_3d_physics::Physics3dPlugin;
use amigo_3d_text::Text3dPlugin;
use amigo_assets::PreparedAssetKind;
use amigo_core::AmigoResult;
use amigo_runtime::{
    PluginBundle, RuntimeBuilder, RuntimePlugin, ServiceRegistry, SystemPhase,
};
use amigo_session::RuntimeSession;

use crate::{LoadedAssetDomainPreparer, LoadedAssetDomainPreparerRegistry};

pub struct ThreeDRuntimeBundle;

impl PluginBundle for ThreeDRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-3d-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(Physics3dPlugin)?
            .with_plugin(MeshPlugin)?
            .with_plugin(ThreeDMeshAssetPreparerPlugin)?
            .with_plugin(Text3dPlugin)?
            .with_plugin(MaterialPlugin)
    }
}

struct ThreeDMeshAssetPreparerPlugin;

struct MeshLoadedAssetPreparer {
    meshes: Arc<MeshSceneService>,
}

impl LoadedAssetDomainPreparer for MeshLoadedAssetPreparer {
    fn name(&self) -> &'static str {
        "amigo-3d-mesh-gltf-preparer"
    }

    fn prepare(&self, catalog: &amigo_assets::AssetCatalog, key: &amigo_assets::AssetKey) {
        let Some(prepared) = catalog.prepared_asset(key) else {
            return;
        };
        if !matches!(prepared.kind, PreparedAssetKind::Mesh3d)
            || !prepared
                .resolved_path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
        {
            return;
        }
        if self
            .meshes
            .request_source_geometry(key.clone(), &prepared.resolved_path)
        {
            // The asset is parsed asynchronously. Keep it in the catalog's
            // external-loading set until the mesh service reports completion.
            catalog.begin_external_prepare(key.clone());
        }
    }

    fn poll(&self, catalog: &amigo_assets::AssetCatalog) {
        for (key, result) in self.meshes.poll_source_geometry() {
            match result {
                Ok(()) => {
                    if let Some(prepared) = catalog.prepared_asset(&key) {
                        // Re-publishing the prepared record removes the
                        // external-loading marker while retaining metadata.
                        catalog.mark_prepared(prepared);
                    }
                }
                Err(reason) => {
                    catalog.mark_failed(key, format!("failed to import GLB mesh: {reason}"));
                }
            }
        }
    }
}

impl RuntimePlugin for ThreeDMeshAssetPreparerPlugin {
    fn name(&self) -> &'static str {
        "amigo-3d-mesh-asset-preparer"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let preparers = registry.required::<LoadedAssetDomainPreparerRegistry>()?;
        let meshes = registry.required::<MeshSceneService>()?;
        preparers.register(Arc::new(MeshLoadedAssetPreparer { meshes }));
        registry.required::<amigo_runtime::SystemRegistry>()?.register_fn(
            SystemPhase::RenderExtract,
            "amigo-3d-poll-asset-preparers",
            |runtime| {
                let catalog = runtime.required::<amigo_assets::AssetCatalog>()?;
                let preparers = runtime.required::<LoadedAssetDomainPreparerRegistry>()?;
                preparers.poll_all(catalog.as_ref());
                Ok(())
            },
        );
        Ok(())
    }
}

pub fn register_three_d_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_3d_physics::register_physics3d_runtime_capabilities(session);
    amigo_3d_mesh::register_mesh3d_runtime_capabilities(session);
    amigo_3d_material::register_material3d_runtime_capabilities(session);
    amigo_3d_text::register_text3d_runtime_capabilities(session);
}
