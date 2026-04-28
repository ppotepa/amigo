use std::sync::Arc;

use amigo_2d_sprite::SpriteSceneService;
use amigo_assets::AssetCatalog;
use amigo_core::{LaunchSelection, RuntimeDiagnostics};
use amigo_input_api::InputState;
use amigo_modding::ModCatalog;
use amigo_scene::SceneService;
use amigo_scripting_api::{DevConsoleQueue, ScriptCommandQueue, ScriptEventQueue};

use crate::bindings::assets::AssetsApi;
use crate::bindings::debug::DebugApi;
use crate::bindings::entities::EntitiesApi;
use crate::bindings::input::InputApi;
use crate::bindings::material3d::Material3dApi;
use crate::bindings::mesh3d::Mesh3dApi;
use crate::bindings::mod_api::ModApi;
use crate::bindings::runtime::RuntimeApi;
use crate::bindings::scene::SceneApi;
use crate::bindings::sprite2d::Sprite2dApi;
use crate::bindings::text2d::Text2dApi;
use crate::bindings::text3d::Text3dApi;
use crate::bindings::time::{ScriptTimeState, TimeApi};
use crate::bindings::ui::UiApi;

#[derive(Clone)]
pub struct WorldApi {
    scene: SceneApi,
    entities: EntitiesApi,
    input: InputApi,
    time: TimeApi,
    assets: AssetsApi,
    mod_api: ModApi,
    sprite2d: Sprite2dApi,
    text2d: Text2dApi,
    mesh3d: Mesh3dApi,
    material3d: Material3dApi,
    text3d: Text3dApi,
    ui: UiApi,
    debug: DebugApi,
    runtime: RuntimeApi,
}

impl WorldApi {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
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
            entities: EntitiesApi { scene },
            input: InputApi { input_state },
            time: TimeApi { state: time_state },
            assets: AssetsApi {
                asset_catalog,
                command_queue: command_queue.clone(),
            },
            mod_api: ModApi {
                launch_selection: launch_selection.clone(),
                mod_catalog,
            },
            sprite2d: Sprite2dApi {
                sprite_scene,
                launch_selection: launch_selection.clone(),
                command_queue: command_queue.clone(),
            },
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
            ui: UiApi {
                command_queue: command_queue.clone(),
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

    pub fn time(&mut self) -> TimeApi {
        self.time.clone()
    }

    pub fn assets(&mut self) -> AssetsApi {
        self.assets.clone()
    }

    pub fn game_mod(&mut self) -> ModApi {
        self.mod_api.clone()
    }

    pub fn sprite2d(&mut self) -> Sprite2dApi {
        self.sprite2d.clone()
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
