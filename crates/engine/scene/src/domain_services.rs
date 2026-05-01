use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Vec2};

use crate::*;

#[derive(Debug, Default)]
pub struct CameraFollow2dSceneService {
    commands: Mutex<Vec<CameraFollow2dSceneCommand>>,
}

impl CameraFollow2dSceneService {
    pub fn queue(&self, command: CameraFollow2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<CameraFollow2dSceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn follow(&self, entity_name: &str) -> Option<CameraFollow2dSceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands
            .iter()
            .find(|command| command.entity_name == entity_name)
            .cloned()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Parallax2dSceneService {
    commands: Mutex<Vec<Parallax2dSceneCommand>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPoolSnapshot {
    pub pool: String,
    pub members: Vec<String>,
    pub active_members: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct EntityPoolState {
    members: BTreeMap<String, Vec<String>>,
    active_members: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Default)]
pub struct EntityPoolSceneService {
    state: Mutex<EntityPoolState>,
}

impl EntityPoolSceneService {
    pub fn queue(&self, command: EntityPoolSceneCommand) {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let active_members = state
            .active_members
            .remove(&command.pool)
            .unwrap_or_default()
            .into_iter()
            .filter(|member| command.members.iter().any(|candidate| candidate == member))
            .collect();
        state.members.insert(command.pool.clone(), command.members);
        state.active_members.insert(command.pool, active_members);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        state.members.clear();
        state.active_members.clear();
    }

    pub fn members(&self, pool: &str) -> Vec<String> {
        self.state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned")
            .members
            .get(pool)
            .cloned()
            .unwrap_or_default()
    }

    pub fn active_members(&self, pool: &str) -> Vec<String> {
        self.state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned")
            .active_members
            .get(pool)
            .map(|members| members.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn active_count(&self, pool: &str) -> usize {
        self.state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned")
            .active_members
            .get(pool)
            .map(BTreeSet::len)
            .unwrap_or(0)
    }

    pub fn snapshot(&self, pool: &str) -> Option<EntityPoolSnapshot> {
        let state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        state.members.get(pool).map(|members| EntityPoolSnapshot {
            pool: pool.to_owned(),
            members: members.clone(),
            active_members: state
                .active_members
                .get(pool)
                .map(|members| members.iter().cloned().collect())
                .unwrap_or_default(),
        })
    }

    pub fn acquire(&self, scene_service: &SceneService, pool: &str) -> Option<String> {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let members = state.members.get(pool)?;
        let free_member = members
            .iter()
            .find(|member| {
                !state
                    .active_members
                    .get(pool)
                    .map(|active| active.contains(*member))
                    .unwrap_or(false)
            })
            .cloned()?;
        state
            .active_members
            .entry(pool.to_owned())
            .or_default()
            .insert(free_member.clone());
        drop(state);
        let _ = scene_service.set_visible(&free_member, true);
        let _ = scene_service.set_simulation_enabled(&free_member, true);
        let _ = scene_service.set_collision_enabled(&free_member, true);
        Some(free_member)
    }

    pub fn release(&self, scene_service: &SceneService, pool: &str, entity_name: &str) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let Some(members) = state.members.get(pool) else {
            return false;
        };
        if !members.iter().any(|member| member == entity_name) {
            return false;
        }
        let was_active = state
            .active_members
            .entry(pool.to_owned())
            .or_default()
            .remove(entity_name);
        drop(state);
        let _ = scene_service.set_visible(entity_name, false);
        let _ = scene_service.set_simulation_enabled(entity_name, false);
        let _ = scene_service.set_collision_enabled(entity_name, false);
        was_active
    }

    pub fn release_all(&self, scene_service: &SceneService, pool: &str) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let Some(active_members) = state.active_members.get_mut(pool) else {
            return 0;
        };
        let released: Vec<String> = active_members.iter().cloned().collect();
        active_members.clear();
        drop(state);

        for entity_name in &released {
            let _ = scene_service.set_visible(entity_name, false);
            let _ = scene_service.set_simulation_enabled(entity_name, false);
            let _ = scene_service.set_collision_enabled(entity_name, false);
        }

        released.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeState {
    pub entity_name: String,
    pub duration_seconds: f32,
    pub remaining_seconds: f32,
    pub outcome: LifetimeExpirationOutcome,
}

#[derive(Debug, Default)]
pub struct LifetimeSceneService {
    definitions: Mutex<BTreeMap<String, LifetimeState>>,
    lifetimes: Mutex<BTreeMap<String, LifetimeState>>,
}

impl LifetimeSceneService {
    pub fn queue(&self, command: LifetimeSceneCommand) {
        let lifetime = LifetimeState {
            entity_name: command.entity_name,
            duration_seconds: command.seconds.max(0.0),
            remaining_seconds: command.seconds.max(0.0),
            outcome: command.outcome,
        };
        self.definitions
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .insert(lifetime.entity_name.clone(), lifetime.clone());
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .insert(lifetime.entity_name.clone(), lifetime);
    }

    pub fn clear(&self) {
        self.definitions
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .clear();
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .clear();
    }

    pub fn lifetime(&self, entity_name: &str) -> Option<LifetimeState> {
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .get(entity_name)
            .cloned()
    }

    pub fn reset_lifetime(&self, entity_name: &str) -> bool {
        let Some(mut lifetime) = self
            .definitions
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .get(entity_name)
            .cloned()
        else {
            return false;
        };
        lifetime.remaining_seconds = lifetime.duration_seconds;
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .insert(entity_name.to_owned(), lifetime);
        true
    }

    pub fn tick(&self, delta_seconds: f32) -> Vec<LifetimeState> {
        let mut lifetimes = self
            .lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned");
        let mut expired = Vec::new();
        let mut expired_names = Vec::new();
        for (entity_name, lifetime) in lifetimes.iter_mut() {
            lifetime.remaining_seconds -= delta_seconds.max(0.0);
            if lifetime.remaining_seconds <= 0.0 {
                expired.push(lifetime.clone());
                expired_names.push(entity_name.clone());
            }
        }
        for entity_name in expired_names {
            lifetimes.remove(&entity_name);
        }
        expired
    }
}

impl Parallax2dSceneService {
    pub fn queue(&self, command: Parallax2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<Parallax2dSceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn set_camera_origin(&self, entity_name: &str, camera_origin: Vec2) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        command.camera_origin = Some(camera_origin);
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

impl Material3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        label: impl Into<String>,
        source: Option<AssetKey>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            label: label.into(),
            albedo: ColorRgba::WHITE,
            source,
        }
    }
}

