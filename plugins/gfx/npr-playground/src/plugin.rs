use crate::render::{NprPlaygroundRenderProfile, NprPlaygroundRenderService};
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_input_api::{InputState, KeyCode, MouseButton};
use amigo_render_npr::{NprCamera, NprDebugView, OrthographicCamera, PerspectiveCamera};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use glam::Vec3;
use std::sync::Mutex;

const DEFAULT_CAMERA_DISTANCE: f32 = 5.0;
const MIN_CAMERA_DISTANCE: f32 = 1.5;
const MAX_CAMERA_DISTANCE: f32 = 20.0;
const MAX_CAMERA_PITCH: f32 = 85.0_f32.to_radians();
// Input wheel values are normalized to pixel-like units by the window host.
// A mouse-wheel notch is roughly 38 units, so use a small response factor
// instead of treating every notch as a whole zoom state.
const ZOOM_SCROLL_RESPONSE: f32 = 0.0035;
const ZOOM_RESPONSE_PER_SECOND: f32 = 6.0;
const AUTO_ROTATION_RADIANS_PER_SECOND: f32 = 0.72;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprPlaygroundProjection {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprPlaygroundStyleProfile {
    MinimalInk,
    PencilAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprPlaygroundSnapshot {
    pub object_position: Vec3,
    pub object_pitch: f32,
    pub object_yaw: f32,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub camera_distance: f32,
    pub projection: NprPlaygroundProjection,
    pub style_profile: NprPlaygroundStyleProfile,
    pub debug_view: NprDebugView,
    pub auto_rotate: bool,
    pub elapsed_seconds: f32,
}

impl Default for NprPlaygroundSnapshot {
    fn default() -> Self {
        Self {
            object_position: Vec3::ZERO,
            object_pitch: 0.0,
            object_yaw: 0.0,
            orbit_yaw: 0.0,
            orbit_pitch: 0.0,
            camera_distance: DEFAULT_CAMERA_DISTANCE,
            projection: NprPlaygroundProjection::Perspective,
            // The playground is a pencil-animation study. The raw ink
            // profile remains available on `3`, but must never be mistaken
            // for the drawing result shown on first launch.
            style_profile: NprPlaygroundStyleProfile::PencilAnimation,
            debug_view: NprDebugView::Final,
            auto_rotate: true,
            elapsed_seconds: 0.0,
        }
    }
}

impl NprPlaygroundSnapshot {
    fn camera(self, aspect: f32) -> NprCamera {
        let direction = Vec3::new(
            self.orbit_yaw.sin() * self.orbit_pitch.cos(),
            self.orbit_pitch.sin(),
            self.orbit_yaw.cos() * self.orbit_pitch.cos(),
        );
        let position = direction * self.camera_distance;
        let perspective = PerspectiveCamera {
            position,
            forward: (-position).normalize_or_zero(),
            up: Vec3::Y,
            vertical_fov: 45.0_f32.to_radians(),
            near: 0.05,
            aspect,
        };
        match self.projection {
            NprPlaygroundProjection::Perspective => NprCamera::Perspective(perspective),
            NprPlaygroundProjection::Orthographic => NprCamera::Orthographic(OrthographicCamera {
                position,
                forward: perspective.forward,
                up: perspective.up,
                vertical_span: 4.0 * self.camera_distance / DEFAULT_CAMERA_DISTANCE,
                near: perspective.near,
                aspect,
            }),
        }
    }
}

#[derive(Debug)]
pub struct NprPlaygroundState {
    snapshot: Mutex<NprPlaygroundSnapshot>,
    target_camera_distance: Mutex<f32>,
    last_cursor_position: Mutex<Option<(f32, f32)>>,
}

impl Default for NprPlaygroundState {
    fn default() -> Self {
        Self {
            snapshot: Mutex::new(NprPlaygroundSnapshot::default()),
            target_camera_distance: Mutex::new(DEFAULT_CAMERA_DISTANCE),
            last_cursor_position: Mutex::new(None),
        }
    }
}

impl NprPlaygroundState {
    pub fn update(&self, input: &InputState, delta_seconds: f32, aspect: f32) -> NprPlaygroundSnapshot {
        let mut snapshot = self.snapshot.lock().expect("NPR playground state mutex");
        let dt = delta_seconds.clamp(0.0, 0.1);
        snapshot.elapsed_seconds += dt;
        let angular_speed = 1.7;

        if snapshot.auto_rotate {
            snapshot.object_yaw += AUTO_ROTATION_RADIANS_PER_SECOND * dt;
        }

        if input.is_down(KeyCode::Left) {
            snapshot.object_yaw += angular_speed * dt;
        }
        if input.is_down(KeyCode::Right) {
            snapshot.object_yaw -= angular_speed * dt;
        }
        if input.is_down(KeyCode::Up) {
            snapshot.object_pitch += angular_speed * dt;
        }
        if input.is_down(KeyCode::Down) {
            snapshot.object_pitch -= angular_speed * dt;
        }
        if input.was_pressed(KeyCode::R) {
            snapshot.object_position = Vec3::ZERO;
            snapshot.object_pitch = 0.0;
            snapshot.object_yaw = 0.0;
        }
        if input.was_pressed(KeyCode::Home) {
            snapshot.orbit_yaw = 0.0;
            snapshot.orbit_pitch = 0.0;
            snapshot.camera_distance = DEFAULT_CAMERA_DISTANCE;
            *self
                .target_camera_distance
                .lock()
                .expect("NPR zoom target mutex") = DEFAULT_CAMERA_DISTANCE;
        }
        if input.was_pressed(KeyCode::Digit1) {
            snapshot.projection = NprPlaygroundProjection::Perspective;
        }
        if input.was_pressed(KeyCode::Digit2) {
            snapshot.projection = NprPlaygroundProjection::Orthographic;
        }
        if input.was_pressed(KeyCode::Digit3) {
            snapshot.style_profile = NprPlaygroundStyleProfile::MinimalInk;
        }
        if input.was_pressed(KeyCode::Digit4) {
            snapshot.style_profile = NprPlaygroundStyleProfile::PencilAnimation;
        }
        if input.was_pressed(KeyCode::Digit5) {
            snapshot.debug_view = NprDebugView::FeatureClasses;
        }
        if input.was_pressed(KeyCode::Digit6) {
            snapshot.debug_view = NprDebugView::Final;
        }
        if input.was_pressed(KeyCode::Space) {
            snapshot.auto_rotate = !snapshot.auto_rotate;
        }

        let mut target_camera_distance = self
            .target_camera_distance
            .lock()
            .expect("NPR zoom target mutex");
        *target_camera_distance = (*target_camera_distance
            * (-input.mouse_wheel_delta_y() * ZOOM_SCROLL_RESPONSE).exp())
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
        let response = 1.0 - (-ZOOM_RESPONSE_PER_SECOND * dt).exp();
        snapshot.camera_distance += (*target_camera_distance - snapshot.camera_distance) * response;

        let cursor = input.cursor_position();
        let mut previous_cursor = self
            .last_cursor_position
            .lock()
            .expect("NPR cursor mutex");
        if let (Some((x, y)), Some((last_x, last_y))) = (cursor, *previous_cursor) {
            let delta_x = x - last_x;
            let delta_y = y - last_y;
            if input.is_mouse_down(MouseButton::Left) {
                snapshot.orbit_yaw -= delta_x * 0.012;
                snapshot.orbit_pitch = (snapshot.orbit_pitch - delta_y * 0.012)
                    .clamp(-MAX_CAMERA_PITCH, MAX_CAMERA_PITCH);
            } else if input.is_mouse_down(MouseButton::Right) {
                let movement_scale = snapshot.camera_distance * 0.0025;
                if input.modifiers().shift {
                    snapshot.object_position.y -= delta_y * movement_scale;
                } else {
                    let camera = snapshot.camera(aspect);
                    let right = camera.forward().cross(camera.up()).normalize_or_zero();
                    let up = right.cross(camera.forward()).normalize_or_zero();
                    snapshot.object_position +=
                        right * delta_x * movement_scale - up * delta_y * movement_scale;
                }
            }
        }
        *previous_cursor = cursor;
        *snapshot
    }

    pub fn snapshot(&self) -> NprPlaygroundSnapshot {
        *self.snapshot.lock().expect("NPR playground state mutex")
    }
}

pub struct NprPlaygroundPlugin;

impl RuntimePlugin for NprPlaygroundPlugin {
    fn name(&self) -> &'static str {
        "amigo-npr-playground-plugin"
    }
    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        let mod_root = registry
            .required::<amigo_modding::ModCatalog>()?
            .mod_by_id("npr-playground")
            .ok_or_else(|| {
                amigo_core::AmigoError::Message(
                    "NPR Playground requires the `npr-playground` mod to be loaded".to_owned(),
                )
            })?
            .root_path
            .canonicalize()
            .map_err(|error| {
                amigo_core::AmigoError::Message(format!(
                    "could not resolve the NPR Playground mod directory: {error}"
                ))
            })?;
        let suzanne_path = mod_root
            .join("assets/models/suzanne/Suzanne.gltf");
        registry.register(NprPlaygroundState::default())?;
        registry.register(NprPlaygroundRenderService::with_suzanne_path(suzanne_path))?;
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "npr_playground_update",
            |runtime| {
                let state = runtime.required::<NprPlaygroundState>()?;
                let input = runtime.required::<InputState>()?;
                let (width, height) = input.viewport_size().unwrap_or((1280.0, 720.0));
                let viewport = [width.max(1.0) as u32, height.max(1.0) as u32];
                let snapshot = state.update(
                    input.as_ref(),
                    amigo_session::simulation_delta_seconds(runtime),
                    viewport[0] as f32 / viewport[1] as f32,
                );
                let render = runtime.required::<NprPlaygroundRenderService>()?;
                render
                    .rebuild_suzanne_rotated(
                        viewport,
                        0x4E5052,
                        snapshot.object_pitch,
                        snapshot.object_yaw,
                        snapshot.object_position,
                        snapshot.camera(viewport[0] as f32 / viewport[1] as f32),
                        match snapshot.style_profile {
                            NprPlaygroundStyleProfile::MinimalInk => NprPlaygroundRenderProfile::MinimalInk,
                            NprPlaygroundStyleProfile::PencilAnimation => NprPlaygroundRenderProfile::PencilAnimation,
                        },
                        snapshot.debug_view,
                        snapshot.elapsed_seconds,
                    )
                    .map_err(amigo_core::AmigoError::Message)?;
                Ok(())
            },
        );
        if let Some(ids) = registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            ids.register(crate::render::NPR_PLAYGROUND_EXTRACTOR_ID);
        }
        register_domain_plugin(
            registry,
            "amigo.gfx.npr-playground",
            &["gfx.npr@1"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playground_opens_in_pencil_animation_not_raw_topology_ink() {
        assert_eq!(
            NprPlaygroundSnapshot::default().style_profile,
            NprPlaygroundStyleProfile::PencilAnimation,
        );
    }
}
