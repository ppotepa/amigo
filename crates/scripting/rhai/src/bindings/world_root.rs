use std::sync::Arc;

use amigo_2d_physics::Physics2dSceneService;
use amigo_assets::AssetCatalog;
use amigo_camera_core_plugin::{CameraFocusTarget2dService, CameraService};
use amigo_composite_plugin::PostFx2dService;
use amigo_core::{LaunchSelection, RuntimeDiagnostics};
use amigo_editor_api::{InspectRequest, InspectRequestService};
use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;
use amigo_modding::ModCatalog;
use amigo_particles_2d_plugin::{Particle2dSceneService, ParticlePreset2dService};
use amigo_render_api::RenderFrameStatsService;
use amigo_scene::{EntityPoolSceneService, LifetimeSceneService, SceneService};
use amigo_scripting_api::{
    DevConsoleQueue, ScriptCommandQueue, ScriptEventQueue, ScriptTraceService,
};
use amigo_shutter_motion_plugin::Motion2dSceneService;
use amigo_sprite_2d_plugin::SpriteSceneService;
use amigo_state::{SceneStateService, SceneTimerService, SessionStateService};
use amigo_ui::UiThemeService;
use amigo_vector_2d_plugin::VectorSceneService;

use crate::bindings::actions::ActionsApi;
use crate::bindings::arcade::ArcadeApi;
use crate::bindings::assets::AssetsApi;
use crate::bindings::audio::AudioApi;
use crate::bindings::beacon2d::Beacon2dApi;
use crate::bindings::camera::CameraApi;
use crate::bindings::debug::DebugApi;
use crate::bindings::entities::EntitiesApi;
use crate::bindings::input::InputApi;
use crate::bindings::layered_image2d::LayeredImage2dApi;
use crate::bindings::light2d::Light2dApi;
use crate::bindings::material3d::Material3dApi;
use crate::bindings::mesh3d::Mesh3dApi;
use crate::bindings::mod_api::ModApi;
use crate::bindings::motion::MotionApi;
use crate::bindings::particles::ParticlesApi;
use crate::bindings::physics::PhysicsApi;
use crate::bindings::physics3d::Physics3dApi;
use crate::bindings::pools::PoolsApi;
use crate::bindings::postfx::PostFxApi;
use crate::bindings::projectiles::ProjectilesApi;
use crate::bindings::random::{RandomApi, ScriptRandomState};
use crate::bindings::render2d::Render2dApi;
use crate::bindings::runtime::RuntimeApi;
use crate::bindings::scene::SceneApi;
use crate::bindings::session::SessionApi;
use crate::bindings::sprite2d::Sprite2dApi;
use crate::bindings::state::StateApi;
use crate::bindings::text2d::Text2dApi;
use crate::bindings::text3d::Text3dApi;
use crate::bindings::time::{ScriptTimeState, TimeApi};
use crate::bindings::timers::TimersApi;
use crate::bindings::trace::TraceApi;
use crate::bindings::ui::UiApi;
use crate::bindings::vector2d::Vector2dApi;

#[derive(Clone)]
pub struct WorldApi {
    scene: SceneApi,
    entities: EntitiesApi,
    input: InputApi,
    layered_image2d: LayeredImage2dApi,
    beacon2d: Beacon2dApi,
    render2d: Render2dApi,
    light2d: Light2dApi,
    actions: ActionsApi,
    arcade: ArcadeApi,
    camera: CameraApi,
    physics: PhysicsApi,
    physics3d: Physics3dApi,
    postfx: PostFxApi,
    pools: PoolsApi,
    projectiles: ProjectilesApi,
    random: RandomApi,
    time: TimeApi,
    assets: AssetsApi,
    audio: AudioApi,
    mod_api: ModApi,
    motion: MotionApi,
    particles: ParticlesApi,
    sprite2d: Sprite2dApi,
    state: StateApi,
    session: SessionApi,
    vector2d: Vector2dApi,
    text2d: Text2dApi,
    mesh3d: Mesh3dApi,
    material3d: Material3dApi,
    text3d: Text3dApi,
    timers: TimersApi,
    trace: TraceApi,
    ui: UiApi,
    debug: DebugApi,
    runtime: RuntimeApi,
    pub inspect_requests: Option<Arc<InspectRequestService>>,
}

impl WorldApi {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        vector_scene: Option<Arc<VectorSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        particle_scene: Option<Arc<Particle2dSceneService>>,
        particle_preset_scene: Option<Arc<ParticlePreset2dService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        post_fx: Option<Arc<PostFx2dService>>,
        camera_service: Option<Arc<CameraService>>,
        focus_targets_2d: Option<Arc<CameraFocusTarget2dService>>,
        pool_scene: Option<Arc<EntityPoolSceneService>>,
        lifetime_scene: Option<Arc<LifetimeSceneService>>,
        state_service: Option<Arc<SceneStateService>>,
        session_service: Option<Arc<SessionStateService>>,
        timer_service: Option<Arc<SceneTimerService>>,
        ui_theme_service: Option<Arc<UiThemeService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        input_actions: Option<Arc<InputActionService>>,
        time_state: Arc<ScriptTimeState>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        render_stats: Option<Arc<RenderFrameStatsService>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
        trace_service: Option<Arc<ScriptTraceService>>,
        inspect_requests: Option<Arc<InspectRequestService>>,
    ) -> Self {
        Self {
            scene: SceneApi {
                scene: scene.clone(),
                launch_selection: launch_selection.clone(),
                mod_catalog: mod_catalog.clone(),
                command_queue: command_queue.clone(),
            },
            entities: EntitiesApi {
                scene: scene.clone(),
            },
            input: InputApi {
                input_state: input_state.clone(),
            },
            layered_image2d: LayeredImage2dApi {
                command_queue: command_queue.clone(),
            },
            beacon2d: Beacon2dApi {
                command_queue: command_queue.clone(),
            },
            render2d: Render2dApi {
                command_queue: command_queue.clone(),
            },
            light2d: Light2dApi {
                command_queue: command_queue.clone(),
            },
            actions: ActionsApi {
                actions: input_actions.clone(),
                input_state: input_state.clone(),
            },
            arcade: ArcadeApi {
                actions: input_actions,
                input_state: input_state.clone(),
                motion: motion_scene.clone(),
                particles: particle_scene.clone(),
            },
            camera: CameraApi {
                camera_service,
                focus_targets_2d,
                asset_catalog: asset_catalog.clone(),
            },
            physics: PhysicsApi {
                scene: scene.clone(),
                physics_scene: physics_scene.clone(),
            },
            physics3d: Physics3dApi {
                launch_selection: launch_selection.clone(),
                command_queue: command_queue.clone(),
            },
            postfx: PostFxApi { post_fx },
            pools: PoolsApi {
                scene: scene.clone(),
                pools: pool_scene.clone(),
                lifetimes: lifetime_scene.clone(),
            },
            projectiles: ProjectilesApi {
                scene: scene.clone(),
                motion_scene: motion_scene.clone(),
                physics_scene,
                pools: pool_scene,
                lifetimes: lifetime_scene,
            },
            random: RandomApi {
                state: Arc::new(ScriptRandomState::default()),
            },
            time: TimeApi { state: time_state },
            assets: AssetsApi {
                asset_catalog: asset_catalog.clone(),
                command_queue: command_queue.clone(),
            },
            audio: AudioApi {
                command_queue: command_queue.clone(),
            },
            mod_api: ModApi {
                launch_selection: launch_selection.clone(),
                mod_catalog: mod_catalog.clone(),
            },
            motion: MotionApi { motion_scene },
            particles: ParticlesApi {
                particles: particle_scene,
                presets: particle_preset_scene,
            },
            sprite2d: Sprite2dApi {
                sprite_scene,
                launch_selection: launch_selection.clone(),
                command_queue: command_queue.clone(),
            },
            state: StateApi {
                state: state_service,
            },
            session: SessionApi {
                session: session_service,
            },
            vector2d: Vector2dApi { vector_scene },
            text2d: Text2dApi {
                launch_selection: launch_selection.clone(),
                command_queue: command_queue.clone(),
            },
            mesh3d: Mesh3dApi {
                launch_selection: launch_selection.clone(),
                asset_catalog: asset_catalog.clone(),
                mod_catalog: mod_catalog.clone(),
                command_queue: command_queue.clone(),
            },
            material3d: Material3dApi {
                launch_selection: launch_selection.clone(),
                command_queue: command_queue.clone(),
            },
            text3d: Text3dApi {
                launch_selection: launch_selection.clone(),
                command_queue: command_queue.clone(),
            },
            timers: TimersApi {
                timers: timer_service,
            },
            trace: TraceApi {
                trace: trace_service,
            },
            ui: UiApi {
                command_queue: command_queue.clone(),
                theme_service: ui_theme_service,
            },
            debug: DebugApi {
                command_queue,
                event_queue,
                console_queue,
            },
            runtime: RuntimeApi {
                launch_selection,
                diagnostics,
                render_stats,
            },
            inspect_requests,
        }
    }

    pub fn scene(&mut self) -> SceneApi {
        self.scene.clone()
    }

    pub fn entities(&mut self) -> EntitiesApi {
        self.entities.clone()
    }

    pub fn input(&mut self) -> InputApi {
        self.input.clone()
    }

    pub fn layered_image2d(&mut self) -> LayeredImage2dApi {
        self.layered_image2d.clone()
    }

    pub fn beacon2d(&mut self) -> Beacon2dApi {
        self.beacon2d.clone()
    }

    pub fn render2d(&mut self) -> Render2dApi {
        self.render2d.clone()
    }

    pub fn light2d(&mut self) -> Light2dApi {
        self.light2d.clone()
    }

    pub fn actions(&mut self) -> ActionsApi {
        self.actions.clone()
    }

    pub fn arcade(&mut self) -> ArcadeApi {
        self.arcade.clone()
    }

    pub fn camera(&mut self) -> CameraApi {
        self.camera.clone()
    }

    pub fn physics(&mut self) -> PhysicsApi {
        self.physics.clone()
    }

    pub fn physics3d(&mut self) -> Physics3dApi {
        self.physics3d.clone()
    }

    pub fn postfx(&mut self) -> PostFxApi {
        self.postfx.clone()
    }

    pub fn pools(&mut self) -> PoolsApi {
        self.pools.clone()
    }

    pub fn projectiles(&mut self) -> ProjectilesApi {
        self.projectiles.clone()
    }

    pub fn random(&mut self) -> RandomApi {
        self.random.clone()
    }

    pub fn time(&mut self) -> TimeApi {
        self.time.clone()
    }

    pub fn assets(&mut self) -> AssetsApi {
        self.assets.clone()
    }

    pub fn audio(&mut self) -> AudioApi {
        self.audio.clone()
    }

    pub fn game_mod(&mut self) -> ModApi {
        self.mod_api.clone()
    }

    pub fn motion(&mut self) -> MotionApi {
        self.motion.clone()
    }

    pub fn particles(&mut self) -> ParticlesApi {
        self.particles.clone()
    }

    pub fn sprite2d(&mut self) -> Sprite2dApi {
        self.sprite2d.clone()
    }

    pub fn state(&mut self) -> StateApi {
        self.state.clone()
    }

    pub fn session(&mut self) -> SessionApi {
        self.session.clone()
    }

    pub fn vector2d(&mut self) -> Vector2dApi {
        self.vector2d.clone()
    }

    pub fn text2d(&mut self) -> Text2dApi {
        self.text2d.clone()
    }

    pub fn mesh3d(&mut self) -> Mesh3dApi {
        self.mesh3d.clone()
    }

    pub fn material3d(&mut self) -> Material3dApi {
        self.material3d.clone()
    }

    pub fn text3d(&mut self) -> Text3dApi {
        self.text3d.clone()
    }

    pub fn timers(&mut self) -> TimersApi {
        self.timers.clone()
    }

    pub fn trace(&mut self) -> TraceApi {
        self.trace.clone()
    }

    pub fn ui(&mut self) -> UiApi {
        self.ui.clone()
    }

    pub fn dev(&mut self) -> DebugApi {
        self.debug.clone()
    }

    pub fn runtime(&mut self) -> RuntimeApi {
        self.runtime.clone()
    }

    pub fn request_inspect(&self, request: InspectRequest) -> bool {
        let Some(queue) = &self.inspect_requests else {
            return false;
        };
        queue.request(request);
        true
    }

    pub(crate) fn runtime_capabilities(&self) -> Vec<String> {
        crate::bindings::runtime::runtime_capabilities(self.runtime.diagnostics.as_ref())
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<WorldApi>("World")
        .register_get("scene", WorldApi::scene)
        .register_get("entities", WorldApi::entities)
        .register_get("input", WorldApi::input)
        .register_get("layered_image2d", WorldApi::layered_image2d)
        .register_get("beacon2d", WorldApi::beacon2d)
        .register_get("render2d", WorldApi::render2d)
        .register_get("light2d", WorldApi::light2d)
        .register_get("actions", WorldApi::actions)
        .register_get("arcade", WorldApi::arcade)
        .register_get("camera", WorldApi::camera)
        .register_get("physics", WorldApi::physics)
        .register_get("physics3d", WorldApi::physics3d)
        .register_get("postfx", WorldApi::postfx)
        .register_get("pools", WorldApi::pools)
        .register_get("projectiles", WorldApi::projectiles)
        .register_get("random", WorldApi::random)
        .register_get("time", WorldApi::time)
        .register_get("assets", WorldApi::assets)
        .register_get("audio", WorldApi::audio)
        .register_get("mod", WorldApi::game_mod)
        .register_get("motion", WorldApi::motion)
        .register_get("particles", WorldApi::particles)
        .register_get("sprite2d", WorldApi::sprite2d)
        .register_get("state", WorldApi::state)
        .register_get("session", WorldApi::session)
        .register_get("vector", WorldApi::vector2d)
        .register_get("text2d", WorldApi::text2d)
        .register_get("mesh3d", WorldApi::mesh3d)
        .register_get("material3d", WorldApi::material3d)
        .register_get("text3d", WorldApi::text3d)
        .register_get("timers", WorldApi::timers)
        .register_get("trace", WorldApi::trace)
        .register_get("ui", WorldApi::ui)
        .register_get("dev", WorldApi::dev)
        .register_get("runtime", WorldApi::runtime);
}
