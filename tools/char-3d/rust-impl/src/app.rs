use crate::{
    assets::{ModelKind, built_in, project_path},
    export,
    mesh::Mesh,
    pipeline::{Mark, RenderFrame, compute_frame},
    renderer::GpuRenderer,
    settings,
    state::{AppState, ControlMode},
    ui::{self, UiAction, UiState},
};
use egui::ViewportId;
use egui_wgpu::{RendererOptions, WgpuConfiguration, WgpuSetup, winit::Painter};
use glam::Vec3;
use std::{
    collections::HashSet,
    num::NonZeroU32,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = Char3dApp::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct Char3dApp {
    state: AppState,
    ui_state: UiState,
    ui_window: Option<Arc<Window>>,
    render_window: Option<Arc<Window>>,
    ui_window_id: Option<WindowId>,
    render_window_id: Option<WindowId>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_painter: Option<Painter>,
    renderer: Option<GpuRenderer>,
    mesh: Option<Mesh>,
    frame: Option<RenderFrame>,
    last_error: String,
    dragging: bool,
    last_cursor: Option<PhysicalPosition<f64>>,
    pressed_keys: HashSet<KeyCode>,
    last_tick: Instant,
    next_redraw: Instant,
}

impl Char3dApp {
    fn new() -> Self {
        let state = settings::load();
        Self {
            state,
            ui_state: UiState::default(),
            ui_window: None,
            render_window: None,
            ui_window_id: None,
            render_window_id: None,
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_painter: None,
            renderer: None,
            mesh: None,
            frame: None,
            last_error: String::new(),
            dragging: false,
            last_cursor: None,
            pressed_keys: HashSet::new(),
            last_tick: Instant::now(),
            next_redraw: Instant::now(),
        }
    }

    fn load_builtin(&mut self, id: &str) {
        let model = built_in(id);
        self.state.model_source = model.id.to_owned();
        match model.kind {
            ModelKind::Obj => {
                let path = project_path(model.path);
                self.load_obj(path, model.label.to_owned());
                self.state.reset_view_for_obj();
            }
            ModelKind::Fbx => {
                let path = project_path(model.path);
                self.state.reset_view_for_fbx();
                self.load_fbx(path, model.label.to_owned());
            }
            ModelKind::AnimClip => {
                let path = project_path(model.path);
                self.state.reset_view_for_fbx();
                self.load_anim_clip(path, model.label.to_owned());
            }
        }
    }

    fn load_obj(&mut self, path: PathBuf, name: String) {
        match Mesh::from_obj_file(&path, name) {
            Ok(mesh) => {
                self.state.reset_animation();
                self.mesh = Some(mesh);
                self.last_error.clear();
                self.invalidate_frame();
            }
            Err(err) => {
                self.last_error = format!("{err}");
            }
        }
    }

    fn load_fbx(&mut self, path: PathBuf, name: String) {
        match Mesh::from_fbx_file(&path, name) {
            Ok(mesh) => {
                self.state.reset_animation();
                self.mesh = Some(mesh);
                self.last_error.clear();
                self.invalidate_frame();
            }
            Err(err) => {
                self.last_error = format!("{err}");
            }
        }
    }

    fn load_anim_clip(&mut self, path: PathBuf, name: String) {
        match Mesh::from_anim_clip_file(&path, name) {
            Ok(mesh) => {
                self.state.reset_animation();
                self.mesh = Some(mesh);
                self.last_error.clear();
                self.invalidate_frame();
            }
            Err(err) => {
                self.last_error = format!("{err}");
            }
        }
    }

    fn invalidate_frame(&mut self) {
        self.frame = None;
        if let Some(window) = &self.render_window {
            window.request_redraw();
        }
    }

    fn ensure_frame(&mut self) {
        let Some(window) = &self.render_window else {
            return;
        };
        let Some(mesh) = &self.mesh else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.frame = Some(compute_frame(mesh, &self.state, size.width, size.height));
    }

    fn redraw_ui(&mut self) {
        let (Some(window), Some(egui_state), Some(painter)) = (
            self.ui_window.as_ref(),
            self.egui_state.as_mut(),
            self.egui_painter.as_mut(),
        ) else {
            return;
        };
        let raw_input = egui_state.take_egui_input(window);
        let before = serde_json::to_string(&self.state).unwrap_or_default();
        let stats = self.frame.as_ref().map(|frame| &frame.stats);
        let mut action = UiAction::None;
        let output = self.egui_ctx.run_ui(raw_input, |ui| {
            action = ui::show(
                ui,
                &mut self.state,
                &mut self.ui_state,
                stats,
                &self.last_error,
            );
        });
        egui_state.handle_platform_output(window, output.platform_output);
        let pixels_per_point = self.egui_ctx.pixels_per_point();
        let clipped = self.egui_ctx.tessellate(output.shapes, pixels_per_point);
        painter.paint_and_update_textures(
            ViewportId::ROOT,
            pixels_per_point,
            [0.965, 0.949, 0.91, 1.0],
            &clipped,
            &output.textures_delta,
            Vec::new(),
        );
        match action {
            UiAction::None => {}
            UiAction::LoadBuiltIn(id) => self.load_builtin(&id),
            UiAction::LoadObjFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("OBJ", &["obj"])
                    .pick_file()
                {
                    let name = path
                        .file_name()
                        .and_then(|v| v.to_str())
                        .unwrap_or("custom.obj")
                        .to_owned();
                    self.load_obj(path, name);
                }
            }
            UiAction::LoadFbxFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("FBX", &["fbx"])
                    .pick_file()
                {
                    let name = path
                        .file_name()
                        .and_then(|v| v.to_str())
                        .unwrap_or("custom.fbx")
                        .to_owned();
                    self.state.model_source = "walking".to_owned();
                    self.state.reset_view_for_fbx();
                    self.load_fbx(path, name);
                }
            }
            UiAction::ResetView => {
                if self.is_fbx_active() {
                    self.state.reset_view_for_fbx();
                } else {
                    self.state.reset_view_for_obj();
                }
                settings::save(&self.state);
                self.invalidate_frame();
            }
            UiAction::ExportSvg => self.export_svg(),
            UiAction::ExportPng => self.export_png(),
            UiAction::ExportAtlas => self.export_atlas(),
        }
        if before != serde_json::to_string(&self.state).unwrap_or_default() {
            settings::save(&self.state);
            self.invalidate_frame();
            self.redraw_renderer();
        }
    }

    fn redraw_renderer(&mut self) {
        if self.frame.is_none() {
            self.ensure_frame();
        }
        if let (Some(renderer), Some(frame)) = (self.renderer.as_mut(), self.frame.as_ref())
            && !renderer.render(frame)
            && let Some(window) = &self.render_window
        {
            window.request_redraw();
        }
    }

    fn export_svg(&mut self) {
        if self.frame.is_none() {
            self.ensure_frame();
        }
        let Some(frame) = &self.frame else {
            self.last_error = "No frame to export.".to_owned();
            return;
        };
        let svg = build_svg(frame);
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("susan_shadow_editor_rust.svg");
        match std::fs::write(&path, svg) {
            Ok(()) => self.last_error = format!("Exported SVG: {}", path.display()),
            Err(err) => self.last_error = format!("SVG export failed: {err}"),
        }
    }

    fn export_png(&mut self) {
        if self.frame.is_none() {
            self.ensure_frame();
        }
        let Some(frame) = &self.frame else {
            self.last_error = "No frame to export.".to_owned();
            return;
        };
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("susan_shadow_editor_rust.png");
        match export::save_png(frame, &path) {
            Ok(()) => self.last_error = format!("Exported PNG: {}", path.display()),
            Err(err) => self.last_error = format!("PNG export failed: {err}"),
        }
    }

    fn export_atlas(&mut self) {
        let Some(mesh) = &self.mesh else {
            self.last_error = "No mesh to export atlas.".to_owned();
            return;
        };
        let size = self
            .render_window
            .as_ref()
            .map(|window| window.inner_size())
            .unwrap_or(PhysicalSize::new(1280, 860));
        let saved = self.state.clone();
        let keys = [
            "cleanInk",
            "engraving",
            "loosePencil",
            "manga",
            "pipelineCleanInk",
            "largeSceneBalanced",
        ];
        let mut frames = Vec::new();
        for key in keys {
            let mut state = saved.clone();
            state.apply_preset(key);
            frames.push((
                key,
                compute_frame(mesh, &state, size.width.max(1), size.height.max(1)),
            ));
        }
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("susan_shadow_style_atlas_rust.png");
        match export::save_atlas(&frames, &path) {
            Ok(()) => self.last_error = format!("Exported atlas: {}", path.display()),
            Err(err) => self.last_error = format!("Atlas export failed: {err}"),
        }
    }

    fn handle_renderer_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state, button, .. } if *button == MouseButton::Left => {
                self.dragging = *state == ElementState::Pressed;
                if !self.dragging {
                    self.last_cursor = None;
                    settings::save(&self.state);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.dragging {
                    if let Some(last) = self.last_cursor {
                        let dx = (position.x - last.x) as f32;
                        let dy = (position.y - last.y) as f32;
                        match self.state.control_mode {
                            ControlMode::Orbit => {
                                self.state.raw_yaw += dx * 0.45;
                                self.state.raw_pitch += dy * 0.35;
                                self.state.yaw = self.state.raw_yaw;
                                self.state.pitch = self.state.raw_pitch.clamp(-85.0, 85.0);
                            }
                            ControlMode::Freelook => {
                                self.state.raw_camera_yaw -= dx * 0.28;
                                self.state.raw_camera_pitch -= dy * 0.24;
                                self.state.camera_yaw = self.state.raw_camera_yaw;
                                self.state.camera_pitch =
                                    self.state.raw_camera_pitch.clamp(-85.0, 85.0);
                            }
                        }
                        self.invalidate_frame();
                    }
                    self.last_cursor = Some(*position);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                if self.state.control_mode == ControlMode::Freelook {
                    self.state.camera_z = (self.state.camera_z - y * 0.01).clamp(-100.0, 100.0);
                } else {
                    self.state.zoom = (self.state.zoom * (-y * 0.001).exp()).clamp(0.55, 1.8);
                }
                settings::save(&self.state);
                self.invalidate_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                match event.state {
                    ElementState::Pressed => {
                        if code == KeyCode::Space {
                            self.state.auto = !self.state.auto;
                            self.invalidate_frame();
                        } else if camera_key(code) {
                            self.state.auto = false;
                            self.pressed_keys.insert(code);
                        }
                    }
                    ElementState::Released => {
                        if self.pressed_keys.remove(&code) {
                            settings::save(&self.state);
                        }
                    }
                }
            }
            WindowEvent::Focused(false) => {
                if !self.pressed_keys.is_empty() {
                    self.pressed_keys.clear();
                    settings::save(&self.state);
                }
            }
            _ => {}
        }
    }

    fn is_fbx_active(&self) -> bool {
        self.mesh
            .as_ref()
            .is_some_and(|mesh| mesh.source_type == "fbx")
            || self.state.model_source == "walking"
    }

    fn update_camera_from_keys(&mut self, dt: f32) -> bool {
        if self.state.control_mode != ControlMode::Freelook || self.pressed_keys.is_empty() {
            return false;
        }
        let mut right = 0.0;
        let mut up = 0.0;
        let mut forward = 0.0;
        let mut look_x = 0.0;
        let mut look_y = 0.0;
        if self.pressed_keys.contains(&KeyCode::KeyD) {
            right += 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::KeyA) {
            right -= 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::KeyE) {
            up += 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::KeyQ) {
            up -= 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::KeyW) {
            forward += 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::KeyS) {
            forward -= 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::ArrowRight) {
            look_x += 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::ArrowLeft) {
            look_x -= 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::ArrowDown) {
            look_y += 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::ArrowUp) {
            look_y -= 1.0;
        }
        if right == 0.0 && up == 0.0 && forward == 0.0 && look_x == 0.0 && look_y == 0.0 {
            return false;
        }
        let fast = if self.pressed_keys.contains(&KeyCode::ShiftLeft)
            || self.pressed_keys.contains(&KeyCode::ShiftRight)
        {
            3.0
        } else {
            1.0
        };
        let move_speed = 8.0 * fast * dt.clamp(0.0, 0.08);
        let yaw = self.state.camera_yaw.to_radians();
        let pitch = self.state.camera_pitch.to_radians();
        let fwd = Vec3::new(
            yaw.sin() * pitch.cos(),
            -pitch.sin(),
            -yaw.cos() * pitch.cos(),
        );
        let rgt = Vec3::new(yaw.cos(), 0.0, yaw.sin());
        let delta = (rgt * right + fwd * forward + Vec3::Y * up) * move_speed;
        self.state.camera_x = (self.state.camera_x + delta.x).clamp(-100.0, 100.0);
        self.state.camera_y = (self.state.camera_y + delta.y).clamp(-100.0, 100.0);
        self.state.camera_z = (self.state.camera_z + delta.z).clamp(-100.0, 100.0);
        self.state.raw_camera_yaw =
            wrap_angle(self.state.raw_camera_yaw + look_x * 100.0 * dt * fast);
        self.state.raw_camera_pitch =
            (self.state.raw_camera_pitch + look_y * 100.0 * dt * fast).clamp(-85.0, 85.0);
        self.state.camera_yaw = snap_angle(self.state.raw_camera_yaw, self.state.angle_snap);
        self.state.camera_pitch = snap_pitch(self.state.raw_camera_pitch, self.state.angle_snap);
        true
    }
}

pub(crate) fn build_svg(frame: &RenderFrame) -> String {
    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        frame.width, frame.height, frame.width, frame.height
    ));
    svg.push_str(&format!(
        r#"<rect width="100%" height="100%" fill="rgb({},{},{})"/>"#,
        (frame.paper[0].clamp(0.0, 1.0) * 255.0) as u8,
        (frame.paper[1].clamp(0.0, 1.0) * 255.0) as u8,
        (frame.paper[2].clamp(0.0, 1.0) * 255.0) as u8
    ));
    svg.push_str(
        "<metadata>{\"tool\":\"char-3d-rust-impl\",\"pipeline\":\"wgpu-vector-npr\"}</metadata>",
    );
    svg.push_str(r#"<g id="paint-regions">"#);
    for region in &frame.paint_regions {
        if region.points.len() < 3 {
            continue;
        }
        svg.push_str(&format!(
            r#"<path d="{}" fill="{}" fill-opacity="{:.3}"/>"#,
            svg_poly_path(&region.points, true),
            svg_color(region.color),
            region.alpha.clamp(0.0, 1.0)
        ));
    }
    svg.push_str(
        r#"</g><g id="shadow-strokes" fill="none" stroke-linecap="round" stroke-linejoin="round">"#,
    );
    for mark in &frame.marks {
        match mark {
            Mark::Line {
                pts,
                color,
                width,
                alpha,
            } => {
                if pts.len() >= 2 {
                    svg.push_str(&format!(
                        r#"<path d="{}" stroke="{}" stroke-opacity="{:.3}" stroke-width="{:.2}"/>"#,
                        svg_poly_path(pts, false),
                        svg_color(*color),
                        alpha.clamp(0.0, 1.0),
                        width
                    ));
                }
            }
            Mark::Dot {
                center,
                radius,
                color,
                alpha,
            } => {
                svg.push_str(&format!(
                    r#"<circle cx="{:.1}" cy="{:.1}" r="{:.2}" fill="{}" fill-opacity="{:.3}"/>"#,
                    center.x,
                    center.y,
                    radius,
                    svg_color(*color),
                    alpha.clamp(0.0, 1.0)
                ));
            }
        }
    }
    svg.push_str(
        r#"</g><g id="contours" fill="none" stroke-linecap="round" stroke-linejoin="round">"#,
    );
    for segment in &frame.contours {
        svg.push_str(&format!(
            r##"<path d="M {:.1} {:.1} L {:.1} {:.1}" stroke="#17110b" stroke-opacity="{}" stroke-width="1.2"/>"##,
            segment.a.x,
            segment.a.y,
            segment.b.x,
            segment.b.y,
            if segment.visible { "0.82" } else { "0.25" }
        ));
    }
    svg.push_str("</g></svg>");
    svg
}

fn svg_color(color: [f32; 4]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8
    )
}

fn svg_poly_path(points: &[glam::Vec2], close: bool) -> String {
    let Some(first) = points.first() else {
        return String::new();
    };
    let mut out = format!("M {:.1} {:.1}", first.x, first.y);
    for point in &points[1..] {
        out.push_str(&format!(" L {:.1} {:.1}", point.x, point.y));
    }
    if close {
        out.push_str(" Z");
    }
    out
}

impl ApplicationHandler for Char3dApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ui_window.is_some() {
            return;
        }
        let ui_window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("char-3d controls")
                        .with_inner_size(PhysicalSize::new(500, 900)),
                )
                .expect("create egui window"),
        );
        let render_window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("char-3d renderer")
                        .with_inner_size(PhysicalSize::new(1280, 860)),
                )
                .expect("create renderer window"),
        );
        self.ui_window_id = Some(ui_window.id());
        self.render_window_id = Some(render_window.id());
        let config = WgpuConfiguration {
            wgpu_setup: WgpuSetup::from_display_handle(event_loop.owned_display_handle()),
            ..Default::default()
        };
        let mut painter = pollster::block_on(Painter::new(
            self.egui_ctx.clone(),
            config,
            false,
            RendererOptions::default(),
        ));
        pollster::block_on(painter.set_window(ViewportId::ROOT, Some(ui_window.clone())))
            .expect("initialize egui painter");
        self.egui_state = Some(egui_winit::State::new(
            self.egui_ctx.clone(),
            ViewportId::ROOT,
            ui_window.as_ref(),
            Some(ui_window.scale_factor() as f32),
            ui_window.theme(),
            None,
        ));
        self.egui_painter = Some(painter);
        self.renderer = Some(
            pollster::block_on(GpuRenderer::new(render_window.clone()))
                .expect("initialize renderer"),
        );
        self.ui_window = Some(ui_window);
        self.render_window = Some(render_window);
        let initial = self.state.model_source.clone();
        self.load_builtin(&initial);
        if let Some(window) = &self.ui_window {
            window.request_redraw();
        }
        if let Some(window) = &self.render_window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) == self.ui_window_id {
            if let (Some(window), Some(egui_state)) =
                (self.ui_window.as_ref(), self.egui_state.as_mut())
            {
                let response = egui_state.on_window_event(window, &event);
                if response.repaint {
                    window.request_redraw();
                }
            }
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    if let Some(painter) = self.egui_painter.as_mut()
                        && let (Some(w), Some(h)) =
                            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    {
                        painter.on_window_resized(ViewportId::ROOT, w, h);
                    }
                }
                WindowEvent::RedrawRequested => self.redraw_ui(),
                _ => {}
            }
            return;
        }
        if Some(window_id) == self.render_window_id {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.resize(size);
                    }
                    self.invalidate_frame();
                }
                WindowEvent::RedrawRequested => self.redraw_renderer(),
                event => self.handle_renderer_event(&event),
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32();
        self.last_tick = now;
        if self.update_camera_from_keys(dt) {
            self.invalidate_frame();
            if let Some(window) = &self.ui_window {
                window.request_redraw();
            }
        }
        if self.state.auto {
            if self.is_fbx_active() {
                let duration = self
                    .mesh
                    .as_ref()
                    .and_then(|mesh| mesh.animation_duration())
                    .unwrap_or(1.35);
                self.state.advance_animation(dt, duration);
            } else {
                self.state.raw_yaw += dt * 28.0;
                self.state.yaw = self.state.raw_yaw;
            }
            self.invalidate_frame();
            if let Some(window) = &self.ui_window {
                window.request_redraw();
            }
        }
        if now >= self.next_redraw {
            self.next_redraw = now + Duration::from_millis(16);
            self.redraw_renderer();
            if let Some(window) = &self.render_window {
                window.request_redraw();
            }
            if let Some(window) = &self.ui_window {
                window.request_redraw();
            }
        }
    }
}

fn camera_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::KeyW
            | KeyCode::KeyA
            | KeyCode::KeyS
            | KeyCode::KeyD
            | KeyCode::KeyQ
            | KeyCode::KeyE
            | KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::ShiftLeft
            | KeyCode::ShiftRight
    )
}

fn wrap_angle(value: f32) -> f32 {
    let mut out = value;
    while out > 180.0 {
        out -= 360.0;
    }
    while out < -180.0 {
        out += 360.0;
    }
    out
}

fn snap_angle(value: f32, angle_snap: f32) -> f32 {
    if angle_snap > 0.0 {
        wrap_angle((value / angle_snap).round() * angle_snap)
    } else {
        wrap_angle(value)
    }
}

fn snap_pitch(value: f32, angle_snap: f32) -> f32 {
    if angle_snap > 0.0 {
        ((value / angle_snap).round() * angle_snap).clamp(-85.0, 85.0)
    } else {
        value.clamp(-85.0, 85.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_export_contains_expected_layers() {
        let mesh =
            Mesh::from_obj_text("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n", "svg-export").unwrap();
        let mut state = AppState::default();
        state.paint_enabled = true;
        state.shadows_enabled = true;
        state.contours = true;
        let frame = compute_frame(&mesh, &state, 160, 120);

        let svg = build_svg(&frame);

        assert!(svg.starts_with("<svg "));
        assert!(svg.contains(r#"id="paint-regions""#));
        assert!(svg.contains(r#"id="shadow-strokes""#));
        assert!(svg.contains(r#"id="contours""#));
        assert!(svg.ends_with("</g></svg>"));
    }

    #[test]
    fn camera_key_updates_freelook_position() {
        let mut app = Char3dApp::new();
        app.state.control_mode = ControlMode::Freelook;
        let before_z = app.state.camera_z;
        app.pressed_keys.insert(KeyCode::KeyW);

        assert!(app.update_camera_from_keys(0.1));

        assert!(app.state.camera_z < before_z);
    }
}
