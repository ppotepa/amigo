//! 3D mesh scene service for referencing authored geometry.
//! It stores mesh bindings that the renderer resolves into GPU-ready draw data.

use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex, mpsc},
};

use amigo_assets::AssetKey;
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
pub use amigo_render_api::{Mesh3d, MeshDrawCommand, MeshGeometry3d, MeshTopologyEdge3d};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{Mesh3dSceneCommand, SceneEntityId, SceneService};
mod editor_capability;
mod geometry_asset;
pub use geometry_asset::*;
mod render_extraction;
mod reset;
mod runtime_capabilities;
mod scene_command;
mod script_command;
pub use editor_capability::*;
pub use render_extraction::*;
pub use reset::*;
pub use runtime_capabilities::*;
pub use scene_command::*;
pub use script_command::*;

#[derive(Debug, Default)]
pub struct MeshSceneService {
    commands: Mutex<Vec<MeshDrawCommand>>,
    geometry: Mutex<BTreeMap<AssetKey, Arc<MeshGeometry3d>>>,
    assets: Mutex<BTreeMap<AssetKey, Arc<MeshGeometryAsset>>>,
    animation_states: Mutex<BTreeMap<String, MeshAnimationState>>,
    pending_geometry: Mutex<BTreeMap<AssetKey, mpsc::Receiver<Result<MeshGeometryAsset, String>>>>,
}

#[derive(Debug, Clone)]
struct MeshAnimationState {
    clip: String,
    time_seconds: f32,
    speed: f32,
    looped: bool,
    sample_fps: Option<f32>,
}

impl MeshSceneService {
    pub fn queue(&self, command: MeshDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        commands.clear();
        self.animation_states
            .lock()
            .expect("mesh animation mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<MeshDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }

    pub fn load_source_geometry(&self, key: AssetKey, path: &Path) -> Result<(), String> {
        let asset = load_gltf_geometry_source_space(path)?;
        self.store_asset(key, asset);
        Ok(())
    }

    /// Starts GLB import off the host thread. The importer is intentionally
    /// isolated from render extraction because large character files can take
    /// hundreds of milliseconds to decode.
    pub fn request_source_geometry(&self, key: AssetKey, path: &Path) -> bool {
        if self.geometry_for(&key).is_some() {
            return false;
        }
        let mut pending = self
            .pending_geometry
            .lock()
            .expect("mesh pending geometry mutex should not be poisoned");
        if pending.contains_key(&key) {
            return false;
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        pending.insert(key, receiver);
        let path = path.to_owned();
        std::thread::Builder::new()
            .name("amigo-gltf-import".to_owned())
            .spawn(move || {
                let result = load_gltf_geometry_source_space(&path);
                let _ = sender.send(result);
            })
            .expect("GLB importer thread should start");
        true
    }

    /// Completes imports without waiting. Each returned item is either stored
    /// in the geometry cache or carries the importer error for diagnostics.
    pub fn poll_source_geometry(&self) -> Vec<(AssetKey, Result<(), String>)> {
        let mut pending = self
            .pending_geometry
            .lock()
            .expect("mesh pending geometry mutex should not be poisoned");
        let mut completed = Vec::new();
        let keys = pending.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            let Some(receiver) = pending.get(&key) else {
                continue;
            };
            match receiver.try_recv() {
                Ok(Ok(geometry)) => {
                    self.store_asset(key.clone(), geometry);
                    completed.push((key, Ok(())));
                }
                Ok(Err(error)) => completed.push((key, Err(error))),
                Err(mpsc::TryRecvError::Disconnected) => {
                    completed.push((key, Err("GLB importer stopped unexpectedly".to_owned())))
                }
                Err(mpsc::TryRecvError::Empty) => continue,
            }
        }
        for (key, _) in &completed {
            pending.remove(key);
        }
        completed
    }

    fn store_asset(&self, key: AssetKey, asset: MeshGeometryAsset) {
        let mut geometry = MeshGeometry3d {
            positions: asset.positions.clone(),
            reference_positions: asset.positions.clone().into(),
            indices: asset.indices.clone(),
            material_indices: asset.material_indices.clone(),
            material_colors: asset.material_colors.clone().into(),
            topology_edges: Vec::new(),
            triangle_edges: Vec::new(),
        };
        prepare_render_topology(&mut geometry);
        self.assets
            .lock()
            .expect("mesh asset mutex should not be poisoned")
            .insert(key.clone(), Arc::new(asset));
        self.geometry
            .lock()
            .expect("mesh geometry mutex should not be poisoned")
            .insert(key, Arc::new(geometry));
    }

    pub fn play_animation(
        &self,
        entity_name: impl Into<String>,
        clip: impl Into<String>,
        speed: f32,
        looped: bool,
    ) -> Result<(), String> {
        self.play_animation_sampled(entity_name, clip, speed, looped, None)
    }

    pub fn play_animation_sampled(
        &self,
        entity_name: impl Into<String>,
        clip: impl Into<String>,
        speed: f32,
        looped: bool,
        sample_fps: Option<f32>,
    ) -> Result<(), String> {
        if !speed.is_finite() || speed < 0.0 {
            return Err("mesh animation speed must be finite and non-negative".to_owned());
        }
        if sample_fps.is_some_and(|fps| !fps.is_finite() || !(1.0..=60.0).contains(&fps)) {
            return Err("mesh animation sample FPS must be in 1..=60".to_owned());
        }
        let entity_name = entity_name.into();
        let clip = clip.into();
        if !self
            .commands()
            .iter()
            .any(|command| command.entity_name == entity_name)
        {
            return Err(format!("mesh animation entity `{entity_name}` does not exist"));
        }
        let mut states = self
            .animation_states
            .lock()
            .expect("mesh animation mutex should not be poisoned");
        let state = states.entry(entity_name).or_insert_with(|| MeshAnimationState {
            clip: clip.clone(),
            time_seconds: 0.0,
            speed,
            looped,
            sample_fps,
        });
        if state.clip != clip {
            state.clip = clip;
            state.time_seconds = 0.0;
        }
        state.speed = speed;
        state.looped = looped;
        state.sample_fps = sample_fps;
        Ok(())
    }

    pub fn advance_animations(&self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        let assets = self
            .assets
            .lock()
            .expect("mesh asset mutex should not be poisoned");
        let base_geometry = self
            .geometry
            .lock()
            .expect("mesh geometry mutex should not be poisoned");
        let mut states = self
            .animation_states
            .lock()
            .expect("mesh animation mutex should not be poisoned");
        let mut commands = self
            .commands
            .lock()
            .expect("mesh command mutex should not be poisoned");
        for command in commands.iter_mut() {
            let Some(state) = states.get_mut(&command.entity_name) else {
                continue;
            };
            let Some(asset) = assets.get(&command.mesh.mesh_asset) else {
                continue;
            };
            let Some(clip) = asset.animations().iter().find(|clip| clip.name == state.clip) else {
                continue;
            };
            state.time_seconds += delta_seconds * state.speed;
            if state.looped {
                state.time_seconds = state
                    .time_seconds
                    .rem_euclid(clip.duration_seconds.max(0.001));
            } else {
                state.time_seconds = state.time_seconds.min(clip.duration_seconds);
            }
            let sample_time = state.sample_fps.map_or(state.time_seconds, |fps| {
                (state.time_seconds * fps).floor() / fps
            });
            let Ok(frame) = asset.sample_animation(clip.index, sample_time) else {
                continue;
            };
            let Some(reference) = base_geometry.get(&command.mesh.mesh_asset) else {
                continue;
            };
            command.mesh.geometry = Some(Arc::new(MeshGeometry3d {
                positions: frame.positions,
                reference_positions: reference.reference_positions.clone(),
                indices: frame.indices,
                material_indices: frame.material_indices,
                material_colors: reference.material_colors.clone(),
                topology_edges: reference.topology_edges.clone(),
                triangle_edges: reference.triangle_edges.clone(),
            }));
        }
    }

    pub fn geometry_for(&self, key: &AssetKey) -> Option<Arc<MeshGeometry3d>> {
        self.geometry
            .lock()
            .expect("mesh geometry mutex should not be poisoned")
            .get(key)
            .cloned()
    }
}

fn prepare_render_topology(geometry: &mut MeshGeometry3d) {
    let mut edge_lookup = BTreeMap::<(u32, u32), u32>::new();
    geometry.topology_edges.clear();
    geometry.triangle_edges.clear();
    geometry.triangle_edges.reserve(geometry.indices.len() / 3);
    for (face, triangle) in geometry.indices.chunks_exact(3).enumerate() {
        let mut face_edges = [0; 3];
        for (slot, (left, right)) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ]
        .into_iter()
        .enumerate()
        {
            let key = if left <= right {
                (left, right)
            } else {
                (right, left)
            };
            let edge_index = *edge_lookup.entry(key).or_insert_with(|| {
                let index = geometry.topology_edges.len() as u32;
                geometry.topology_edges.push(MeshTopologyEdge3d {
                    vertices: [key.0, key.1],
                    faces: [None, None],
                });
                index
            });
            let edge = &mut geometry.topology_edges[edge_index as usize];
            if edge.faces[0].is_none() {
                edge.faces[0] = Some(face as u32);
            } else if edge.faces[1].is_none() {
                edge.faces[1] = Some(face as u32);
            }
            face_edges[slot] = edge_index;
        }
        geometry.triangle_edges.push(face_edges);
    }
}

#[derive(Debug, Clone)]
pub struct MeshDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct MeshPlugin;

impl RuntimePlugin for MeshPlugin {
    fn name(&self) -> &'static str {
        "amigo-3d-mesh"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(MeshSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, MeshSceneResetHandler)?;
        registry.register(MeshDomainInfo {
            crate_name: "amigo-3d-mesh",
            capability: "rendering_3d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-3d-mesh",
            &["rendering_3d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        plugin_scene_handlers.register(
            amigo_scene::MESH_3D_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Mesh3dSceneCommandHandler),
        );
        plugin_scene_handlers.register(
            amigo_scene::MESH_ANIMATION_3D_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Mesh3dSceneCommandHandler),
        );
        registry.required::<amigo_runtime::SystemRegistry>()?.register_fn(
            amigo_runtime::SystemPhase::Update,
            "mesh_animations",
            |runtime| {
                let dt = amigo_session::simulation_delta_seconds(runtime);
                runtime.required::<MeshSceneService>()?.advance_animations(dt);
                Ok(())
            },
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::Mesh3dScriptCommandHandler,
        );
        Ok(())
    }
}

pub fn queue_mesh_scene_command(
    scene_service: &SceneService,
    mesh_scene_service: &MeshSceneService,
    command: &Mesh3dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    mesh_scene_service.queue(MeshDrawCommand {
        entity_id: entity.raw(),
        entity_name: command.entity_name.clone(),
        mesh: Mesh3d {
            mesh_asset: command.mesh_asset.clone(),
            transform: command.transform,
            geometry: None,
        },
    });
    entity
}

#[cfg(test)]
mod tests {
    use super::{
        Mesh3d, Mesh3dEditorCapability, MeshDrawCommand, MeshGeometry3d, MeshSceneService,
        extract_mesh3d_render_commands, prepare_render_topology, queue_mesh_scene_command,
    };
    use amigo_assets::AssetKey;
    use amigo_editor_api::EditorCapability;
    use amigo_math::{Transform3, Vec3};
    use amigo_scene::{Mesh3dSceneCommand, SceneService};

    #[test]
    fn stores_mesh_draw_commands() {
        let service = MeshSceneService::default();

        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name: "playground-3d-probe".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
                transform: Transform3::default(),
                geometry: None,
            },
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-3d-probe".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }

    #[test]
    fn queues_mesh_scene_command() {
        let scene = SceneService::default();
        let service = MeshSceneService::default();

        let entity = queue_mesh_scene_command(
            &scene,
            &service,
            &Mesh3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                AssetKey::new("playground-3d/meshes/probe"),
            ),
        );

        assert_eq!(entity.raw(), 0);
        assert_eq!(service.commands().len(), 1);
        assert_eq!(scene.entity_names(), vec!["playground-3d-probe".to_owned()]);
    }

    #[test]
    fn extraction_uses_latest_scene_pose_for_animated_meshes() {
        let scene = SceneService::default();
        let service = MeshSceneService::default();
        queue_mesh_scene_command(
            &scene,
            &service,
            &Mesh3dSceneCommand::new(
                "playground-3d",
                "moving-probe",
                AssetKey::new("playground-3d/meshes/probe"),
            ),
        );

        assert!(scene.set_entity_pose_3d(
            "moving-probe",
            Vec3::new(3.0, 1.0, -2.0),
            Vec3::new(0.0, 0.5, 0.0),
        ));
        let commands = extract_mesh3d_render_commands(super::Mesh3dRenderExtractionContext {
            scene_service: &scene,
            mesh_scene_service: &service,
        });

        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0].mesh.transform.translation,
            Vec3::new(3.0, 1.0, -2.0)
        );
        assert_eq!(commands[0].mesh.transform.rotation_euler.y, 0.5);
    }

    #[test]
    fn render_topology_is_prepared_once_per_geometry_asset() {
        let mut geometry = MeshGeometry3d {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            material_indices: vec![0, 1],
            material_colors: vec![[1.0; 4], [0.5, 0.5, 0.5, 1.0]].into(),
            reference_positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]].into(),
            topology_edges: Vec::new(),
            triangle_edges: Vec::new(),
        };

        prepare_render_topology(&mut geometry);

        assert_eq!(geometry.topology_edges.len(), 5);
        assert_eq!(geometry.triangle_edges.len(), 2);
        let shared = geometry
            .topology_edges
            .iter()
            .find(|edge| edge.vertices == [0, 2])
            .expect("diagonal should be shared by both triangles");
        assert_eq!(shared.faces, [Some(0), Some(1)]);
    }

    #[test]
    fn mesh_editor_capability_uses_mesh3d_component_type() {
        let capability = Mesh3dEditorCapability;
        assert_eq!(capability.component_type().as_str(), "amigo.3d.mesh");
        assert_eq!(capability.inspector_schema().fields.len(), 4);
    }
}
