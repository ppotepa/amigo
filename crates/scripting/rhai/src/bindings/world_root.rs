use std::sync::Arc;

use amigo_2d_motion::Motion2dSceneService;
use amigo_2d_particles::Particle2dSceneService;
use amigo_2d_physics::Physics2dSceneService;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_assets::AssetCatalog;
use amigo_core::{LaunchSelection, RuntimeDiagnostics};
use amigo_input_api::InputState;
use amigo_modding::ModCatalog;
use amigo_scene::{EntityPoolSceneService, LifetimeSceneService, SceneService};
use amigo_scripting_api::{DevConsoleQueue, ScriptCommandQueue, ScriptEventQueue};
use amigo_state::{SceneStateService, SceneTimerService, SessionStateService};
use amigo_ui::UiThemeService;

use crate::bindings::assets::AssetsApi;
use crate::bindings::audio::AudioApi;
use crate::bindings::debug::DebugApi;
use crate::bindings::entities::EntitiesApi;
use crate::bindings::input::InputApi;
use crate::bindings::material3d::Material3dApi;
use crate::bindings::mesh3d::Mesh3dApi;
use crate::bindings::mod_api::ModApi;
use crate::bindings::motion::MotionApi;
use crate::bindings::particles::ParticlesApi;
use crate::bindings::physics::PhysicsApi;
use crate::bindings::pools::PoolsApi;
use crate::bindings::projectiles::ProjectilesApi;
use crate::bindings::runtime::RuntimeApi;
use crate::bindings::scene::SceneApi;
use crate::bindings::session::SessionApi;
use crate::bindings::sprite2d::Sprite2dApi;
use crate::bindings::state::StateApi;
use crate::bindings::text2d::Text2dApi;
use crate::bindings::text3d::Text3dApi;
use crate::bindings::time::{ScriptTimeState, TimeApi};
use crate::bindings::timers::TimersApi;
use crate::bindings::ui::UiApi;
use crate::bindings::vector2d::Vector2dApi;

#[derive(Clone)]
pub struct WorldApi {
    scene: SceneApi,
    entities: EntitiesApi,
    input: InputApi,
    physics: PhysicsApi,
    pools: PoolsApi,
    projectiles: ProjectilesApi,
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
    ui: UiApi,
    debug: DebugApi,
    runtime: RuntimeApi,
}

impl WorldApi {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        vector_scene: Option<Arc<VectorSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        particle_scene: Option<Arc<Particle2dSceneService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        pool_scene: Option<Arc<EntityPoolSceneService>>,
        lifetime_scene: Option<Arc<LifetimeSceneService>>,
        state_service: Option<Arc<SceneStateService>>,
        session_service: Option<Arc<SessionStateService>>,
        timer_service: Option<Arc<SceneTimerService>>,
        ui_theme_service: Option<Arc<UiThemeService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        time_state: Arc<ScriptTimeState>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
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
            input: InputApi { input_state },
            physics: PhysicsApi {
                scene: scene.clone(),
                physics_scene: physics_scene.clone(),
            },
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
            time: TimeApi { state: time_state },
            assets: AssetsApi {
                asset_catalog,
                command_queue: command_queue.clone(),
            },
            audio: AudioApi {
                command_queue: command_queue.clone(),
            },
            mod_api: ModApi {
                launch_selection: launch_selection.clone(),
                mod_catalog,
            },
            motion: MotionApi { motion_scene },
            particles: ParticlesApi {
                particles: particle_scene,
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
            },
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

    pub fn physics(&mut self) -> PhysicsApi {
        self.physics.clone()
    }

    pub fn pools(&mut self) -> PoolsApi {
        self.pools.clone()
    }

    pub fn projectiles(&mut self) -> ProjectilesApi {
        self.projectiles.clone()
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

    pub fn ui(&mut self) -> UiApi {
        self.ui.clone()
    }

    pub fn dev(&mut self) -> DebugApi {
        self.debug.clone()
    }

    pub fn runtime(&mut self) -> RuntimeApi {
        self.runtime.clone()
    }
}
