use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use amigo_app::{ScenePreviewHost, ScenePreviewOptions};
use amigo_tool_scene_preview::{
    load_static_scene_preview, PreviewColor, PreviewDrawItem, PreviewRequest, PreviewSceneInfo,
    PreviewSnapshot, PreviewState, ScenePreviewController,
};
use amigo_tool_scene_snapshot::{SceneSnapshotImage, SceneSnapshotMode, SceneSnapshotRequest};
use amigo_modding::requested_mods_for_root;
use eframe::egui;
use egui::{Color32, RichText, Vec2};
use serde::Deserialize;

const APP_TITLE: &str = "Amigo Object Browser";
const PRIMARY: Color32 = Color32::from_rgb(44, 62, 80);
const BODY_BG: Color32 = Color32::from_rgb(248, 249, 250);
const CARD_BG: Color32 = Color32::WHITE;
const BORDER: Color32 = Color32::from_rgb(222, 226, 230);
const TEXT_MUTED: Color32 = Color32::from_rgb(108, 117, 125);
const SUCCESS: Color32 = Color32::from_rgb(24, 188, 156);
const WARNING: Color32 = Color32::from_rgb(243, 156, 18);
const PREVIEW_BG: Color32 = Color32::from_rgb(12, 14, 18);
const PREVIEW_TEXT: Color32 = Color32::WHITE;
const PREVIEW_TEXT_OUTLINE: Color32 = Color32::BLACK;
const NAVBAR_HEIGHT: f32 = 72.0;
const STATUS_HEIGHT: f32 = 34.0;
const BODY_MARGIN: f32 = 24.0;
const SIDEBAR_WIDTH: f32 = 390.0;
const MOD_STATUS_WIDTH: f32 = 76.0;
const DIALOG_PREVIEW_RENDER_WIDTH: u32 = 1280;
const DIALOG_PREVIEW_RENDER_HEIGHT: u32 = 720;
const PREVIEW_FPS: u32 = 5;
const PREVIEW_FRAME_INTERVAL: Duration = Duration::from_millis(1000 / PREVIEW_FPS as u64);

#[derive(Debug, Deserialize)]
struct ModManifest {
    id: String,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    capabilities: Option<Vec<String>>,
    launcher_category: Option<Vec<String>>,
    scenes: Option<Vec<SceneManifest>>,
}

#[derive(Debug, Deserialize)]
struct SceneManifest {
    id: String,
    label: Option<String>,
    description: Option<String>,
    launcher_visible: Option<bool>,
}

#[derive(Clone, Debug)]
struct ModSummary {
    id: String,
    name: String,
    version: String,
    description: String,
    root: PathBuf,
    status: ModStatus,
    categories: Vec<String>,
    capabilities: Vec<String>,
    scenes: Vec<SceneSummary>,
    asset_count: usize,
}

#[derive(Clone, Debug)]
struct SceneSummary {
    id: String,
    label: String,
    description: String,
    launcher_visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreviewPlayback {
    Play,
    Pause,
    Stop,
}

impl PreviewPlayback {
    fn label(self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Pause => "Pause",
            Self::Stop => "Stop",
        }
    }
}

#[derive(Clone, Debug)]
enum ModStatus {
    Ready,
    Warning(String),
    Error(String),
}

impl ModStatus {
    fn label(&self) -> &str {
        match self {
            Self::Ready => "Ready",
            Self::Warning(_) => "Warning",
            Self::Error(_) => "Error",
        }
    }

    fn color(&self) -> Color32 {
        match self {
            Self::Ready => SUCCESS,
            Self::Warning(_) => WARNING,
            Self::Error(_) => Color32::from_rgb(231, 76, 60),
        }
    }

    fn detail(&self) -> Option<&str> {
        match self {
            Self::Ready => None,
            Self::Warning(message) | Self::Error(message) => Some(message.as_str()),
        }
    }
}

struct ObjectBrowserApp {
    mods_root: PathBuf,
    mods: Vec<ModSummary>,
    selected_mod: Option<String>,
    selected_scene: Option<String>,
    playback: PreviewPlayback,
    next_frame_at: Option<Instant>,
    preview: ScenePreviewController,
    preview_texture: Option<PreviewTextureCache>,
    preview_worker: Option<PreviewWorker>,
    preview_capture_in_flight: bool,
    preview_live_token: u64,
    preview_live_scene_key: Option<String>,
    preview_latest_token: u64,
    search: String,
    status: String,
}

struct PreviewTextureCache {
    key: String,
    width: u32,
    height: u32,
    handle: egui::TextureHandle,
}

#[derive(Debug)]
struct PreviewWorker {
    command_tx: Sender<PreviewWorkerCommand>,
    result_rx: Receiver<PreviewWorkerResult>,
    handle: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug)]
enum PreviewWorkerCommand {
    Configure {
        token: u64,
        scene_key: String,
        options: ScenePreviewOptions,
        snapshot_request: SceneSnapshotRequest,
    },
    CaptureNextFrame {
        token: u64,
        scene_key: String,
    },
    Stop,
    Shutdown,
}

#[derive(Debug)]
enum PreviewWorkerResult {
    Rendered {
        token: u64,
        scene_key: String,
        image: SceneSnapshotImage,
    },
    RenderError {
        token: u64,
        scene_key: String,
        message: String,
    },
}

impl ObjectBrowserApp {
    fn new(mods_root: PathBuf) -> Self {
        let mut app = Self {
            mods_root,
            mods: Vec::new(),
            selected_mod: None,
            selected_scene: None,
            playback: PreviewPlayback::Play,
            next_frame_at: Some(Instant::now()),
            preview: ScenePreviewController::new(),
            preview_texture: None,
            preview_worker: Some(PreviewWorker::new()),
            preview_capture_in_flight: false,
            preview_live_token: 0,
            preview_live_scene_key: None,
            preview_latest_token: 0,
            search: String::new(),
            status: String::from("Starting"),
        };
        app.refresh_mods();
        app
    }

    fn refresh_mods(&mut self) {
        self.mods = scan_mods(&self.mods_root);
        self.mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        if self
            .selected_mod
            .as_ref()
            .is_none_or(|selected| !self.mods.iter().any(|m| &m.id == selected))
        {
            self.selected_mod = self.mods.first().map(|m| m.id.clone());
        }
        self.sync_selected_scene();
        self.request_preview_for_current_selection(false);
        self.status = format!("{} mods found in {}", self.mods.len(), self.mods_root.display());
    }

    fn selected(&self) -> Option<&ModSummary> {
        let selected = self.selected_mod.as_ref()?;
        self.mods.iter().find(|m| &m.id == selected)
    }

    fn selected_scene_for_current_mod(&self) -> Option<&SceneSummary> {
        let summary = self.selected()?;
        let selected_scene = self.selected_scene.as_ref()?;
        summary.scenes.iter().find(|scene| &scene.id == selected_scene)
    }

    fn selected_scene_or_fallback(&self) -> Option<SceneSummary> {
        let summary = self.selected()?;
        self.selected_scene_for_current_mod()
            .cloned()
            .or_else(|| preview_scene_for(&summary).cloned())
    }

    fn preview_host_key(summary: &ModSummary, scene_id: &str) -> String {
        format!(
            "{}:{}:{}x{}",
            summary.id,
            scene_id,
            DIALOG_PREVIEW_RENDER_WIDTH,
            DIALOG_PREVIEW_RENDER_HEIGHT
        )
    }

    fn select_mod(&mut self, mod_id: &str) {
        self.selected_mod = Some(mod_id.to_string());
        self.sync_selected_scene();
        self.request_preview_for_current_selection(false);
        self.status = format!("Selected mod: {mod_id}");
    }

    fn select_scene(&mut self, scene_id: &str) {
        let Some(summary) = self.selected() else {
            return;
        };
        let mod_id = summary.id.clone();
        let scene_exists = summary.scenes.iter().any(|scene| scene.id == scene_id);
        if scene_exists {
            self.selected_scene = Some(scene_id.to_string());
            self.request_preview_for_current_selection(false);
            self.status = format!("Selected scene: {mod_id}:{scene_id}");
        }
    }

    fn set_playback(&mut self, playback: PreviewPlayback) {
        if self.playback == playback {
            return;
        }

        self.playback = playback;
        self.next_frame_at = Some(Instant::now() + PREVIEW_FRAME_INTERVAL);

        match self.playback {
            PreviewPlayback::Play => {
                self.preview_capture_in_flight = false;
                self.preview_latest_token = self.preview_latest_token.saturating_add(1);
                self.next_frame_at = Some(Instant::now());
                self.request_preview_for_current_selection(false);
                self.status = String::from("Preview playback started (5 fps)");
            }
            PreviewPlayback::Pause => {
                self.next_frame_at = None;
                self.stop_preview_worker_capture();
                self.status = String::from("Preview paused");
            }
            PreviewPlayback::Stop => {
                self.next_frame_at = None;
                self.preview.clear();
                self.stop_preview_worker_capture();
                self.status = String::from("Preview stopped");
            }
        }
    }

    fn maybe_update_live_preview(&mut self) {
        if self.playback != PreviewPlayback::Play {
            return;
        }

        self.try_apply_live_preview_result();

        let now = Instant::now();
        let ready_for_next = self
            .next_frame_at
            .is_none_or(|scheduled| now >= scheduled);
        if !ready_for_next {
            return;
        }

        if self.preview.state().is_loading() {
            self.next_frame_at = Some(now + PREVIEW_FRAME_INTERVAL);
            return;
        }

        if self.selected().is_some() {
            if self.preview_capture_in_flight {
                return;
            }
            self.request_preview_for_current_selection(false);
            self.next_frame_at = Some(now + PREVIEW_FRAME_INTERVAL);
            return;
        }

        self.next_frame_at = Some(now + PREVIEW_FRAME_INTERVAL);
    }

    fn try_apply_live_preview_result(&mut self) {
        let token = self.preview_live_token;
        let Some(scene_key) = self.preview_live_scene_key.as_deref() else {
            return;
        };

        let mut results = Vec::new();
        let Some(worker) = self.preview_worker.as_ref() else {
            return;
        };
        while let Ok(result) = worker.pull_result() {
            results.push(result);
        }

        let (mod_id, scene_id) = match self.selected() {
            Some(summary) => {
                let scene_id = self
                    .selected_scene_for_current_mod()
                    .or_else(|| preview_scene_for(summary))
                    .map(|scene| scene.id.clone())
                    .unwrap_or_else(|| String::from("unknown"));
                (summary.id.clone(), scene_id)
            }
            None => (String::from("unknown"), String::from("unknown")),
        };

        for result in results {
            match result {
                PreviewWorkerResult::Rendered {
                    token: result_token,
                    scene_key: result_scene_key,
                    image,
                } if result_token == token && result_scene_key == scene_key => {
                    self.preview.show_rendered_immediately(image);
                    self.preview_capture_in_flight = false;
                    self.status = String::from("Live preview frame updated");
                }
                PreviewWorkerResult::RenderError {
                    token: result_token,
                    scene_key: result_scene_key,
                    message,
                } if result_token == token && result_scene_key == scene_key => {
                    self.preview.set_error(
                        Some(PreviewRequest::new(mod_id.clone(), scene_id.clone())),
                        message,
                    );
                    self.preview_capture_in_flight = false;
                }
                _ => {}
            }
        }
    }

    fn stop_preview_worker_capture(&mut self) {
        self.preview_capture_in_flight = false;
        if let Some(worker) = self.preview_worker.as_ref() {
            let _ = worker.command_tx.send(PreviewWorkerCommand::Stop);
        }
    }

    fn request_preview_for_current_selection(&mut self, _immediate: bool) {
        let Some(summary) = self.selected().cloned() else {
            self.preview.clear();
            self.stop_preview_worker_capture();
            return;
        };
        let Some(scene) = self.selected_scene_or_fallback() else {
            self.preview.clear();
            self.stop_preview_worker_capture();
            return;
        };
        let info = PreviewSceneInfo::new(
            summary.id.clone(),
            scene.id.clone(),
            scene.label.clone(),
            summary.scenes.len(),
        );
        let scene_path = summary.root.join("scenes").join(&scene.id).join("scene.yml");

        if self.playback == PreviewPlayback::Stop {
            self.stop_preview_worker_capture();
            self.preview.request_placeholder(info);
            return;
        }

        if self.playback == PreviewPlayback::Play {
            let snapshot_request = SceneSnapshotRequest::new(
                info.mod_id.clone(),
                info.scene_id.clone(),
                DIALOG_PREVIEW_RENDER_WIDTH,
                DIALOG_PREVIEW_RENDER_HEIGHT,
                SceneSnapshotMode::EnginePreview,
            )
            .with_paths(summary.root.clone(), scene_path.clone());

            let host_key = Self::preview_host_key(&summary, &scene.id);
            let (host_key_changed, token) = match &self.preview_live_scene_key {
                Some(current) if current == &host_key => (false, self.preview_live_token),
                _ => {
                    self.preview_live_token = self.preview_live_token.saturating_add(1);
                    (true, self.preview_live_token)
                }
            };
            self.preview_live_token = token;
            self.preview_live_scene_key = Some(host_key.clone());

            let options = ScenePreviewOptions::new(
                mods_root_for_mod(&summary.root),
                summary.id.clone(),
                scene.id.clone(),
                snapshot_request.width,
                snapshot_request.height,
            )
            .with_active_mods(requested_mods_for_root(&summary.id))
            .with_warmup_frames(snapshot_request.warmup_frames())
            .with_playback_delta_seconds(1.0 / PREVIEW_FPS as f32);

            if let Some(worker) = self.preview_worker.as_ref() {
                if host_key_changed {
                    worker.configure(token, host_key.clone(), options, snapshot_request.clone());
                }
                if !self.preview_capture_in_flight {
                    self.preview_capture_in_flight = true;
                    self.status = format!("Rendering live frame [{token}]");
                    worker.capture_next_frame(token, host_key.clone());
                    if host_key_changed || !matches!(self.preview.state(), PreviewState::ReadyRendered { .. }) {
                        self.preview.request_placeholder(info);
                    }
                }
            } else {
                self.preview.set_error(
                    Some(PreviewRequest::new(info.mod_id.clone(), info.scene_id.clone())),
                    "Preview worker unavailable",
                );
                self.stop_preview_worker_capture();
                if let Ok(snapshot) = load_static_scene_preview(&scene_path, info.clone()) {
                    self.preview.request_snapshot(snapshot);
                } else {
                    self.preview.request_placeholder(info);
                }
            }
            return;
        }

        self.stop_preview_worker_capture();
        if !matches!(self.preview.state(), PreviewState::ReadyRendered { .. }) {
            self.preview.request_placeholder(info);
        }
    }

    fn sync_selected_scene(&mut self) {
        let Some(summary) = self.selected() else {
            self.selected_scene = None;
            return;
        };
        if self
            .selected_scene
            .as_ref()
            .is_some_and(|scene_id| summary.scenes.iter().any(|scene| &scene.id == scene_id))
        {
            return;
        }
        let next_scene = preview_scene_for(summary).map(|scene| scene.id.clone());
        self.selected_scene = next_scene;
    }

    fn filtered_mods(&self) -> Vec<ModSummary> {
        let query = self.search.trim().to_lowercase();
        self.mods
            .iter()
            .filter(|m| {
                query.is_empty()
                    || m.name.to_lowercase().contains(&query)
                    || m.id.to_lowercase().contains(&query)
                    || m.description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    fn draw_mod_card(&mut self, ui: &mut egui::Ui, summary: &ModSummary) {
        let selected = self.selected_mod.as_deref() == Some(summary.id.as_str());
        let frame_fill = if selected {
            Color32::from_rgb(235, 248, 246)
        } else {
            CARD_BG
        };
        let stroke = if selected {
            egui::Stroke::new(2.0, PRIMARY)
        } else {
            egui::Stroke::new(1.0, BORDER)
        };

        let mut open_clicked = false;
        let card = egui::Frame::none()
            .fill(frame_fill)
            .stroke(stroke)
            .inner_margin(egui::Margin::same(12.0))
            .rounding(8.0)
            .show(ui, |ui| {
                let title_width = (ui.available_width() - MOD_STATUS_WIDTH - 8.0).max(120.0);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [title_width, 24.0],
                        egui::Label::new(RichText::new(&summary.name).strong().size(18.0)),
                    );
                    ui.add_space(8.0);
                    status_badge(ui, summary.status.label(), summary.status.color(), MOD_STATUS_WIDTH);
                });
                ui.add_space(2.0);
                ui.label(RichText::new(&summary.description).color(TEXT_MUTED).small());
                ui.add_space(4.0);
                ui.label(format!(
                    "{} scenes | {} assets | {}",
                    summary.scenes.len(),
                    summary.asset_count,
                    summary.version
                ));
                if !summary.categories.is_empty() {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for category in &summary.categories {
                            ui.label(RichText::new(category).small().color(PRIMARY));
                        }
                    });
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let can_open = !matches!(summary.status, ModStatus::Error(_));
                    if ui.add_enabled(can_open, egui::Button::new("Open mod")).clicked() {
                        self.select_mod(&summary.id);
                        self.status = format!("Open requested: {}", summary.id);
                        open_clicked = true;
                    }
                    ui.label(RichText::new(if selected { "current context" } else { "click card to inspect" }).small().color(TEXT_MUTED));
                });
            });

        if card.response.clicked() && !open_clicked {
            self.select_mod(&summary.id);
        }
    }

    fn draw_preview_placeholder(&mut self, ui: &mut egui::Ui, summary: &ModSummary, desired: Vec2) {
        let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, PREVIEW_BG);
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, Color32::from_rgb(55, 62, 76)),
        );
        if self.preview.state().is_loading() {
            draw_preview_loading_overlay(ui, rect);
            return;
        }
        let (title, subtitle, scene_count) = match self.preview.state() {
            PreviewState::ReadyRendered { image } => {
                let image = image.clone();
                self.draw_rendered_snapshot(ui, rect, &image);
                return;
            }
            PreviewState::ReadySnapshot { snapshot } => {
                draw_preview_snapshot(ui, rect, snapshot);
                return;
            }
            PreviewState::ReadyPlaceholder { info } => (
                info.scene_label.clone(),
                format!("scene: {} | pregenerated preview placeholder", info.scene_id),
                info.scene_count,
            ),
            PreviewState::Error { message, .. } => {
                draw_preview_error(ui, rect, message);
                return;
            }
            PreviewState::Empty | PreviewState::Loading { .. } => {
                let scene = self
                    .selected_scene_for_current_mod()
                    .or_else(|| preview_scene_for(summary));
                (
                    scene
                        .map(|scene| scene.label.clone())
                        .unwrap_or_else(|| summary.name.clone()),
                    scene
                        .map(|scene| format!("scene: {} | pregenerated preview placeholder", scene.id))
                        .unwrap_or_else(|| String::from("logo / fullscreen preview placeholder")),
                    summary.scenes.len(),
                )
            }
        };
        draw_outlined_text(
            &painter,
            rect.center() - Vec2::new(0.0, 58.0),
            egui::Align2::CENTER_CENTER,
            "Core Runtime Preview",
            egui::FontId::proportional(18.0),
            PREVIEW_TEXT,
        );
        draw_outlined_text(
            &painter,
            rect.center() - Vec2::new(0.0, 18.0),
            egui::Align2::CENTER_CENTER,
            title,
            egui::FontId::proportional(38.0),
            PREVIEW_TEXT,
        );
        draw_outlined_text(
            &painter,
            rect.center() + Vec2::new(0.0, 26.0),
            egui::Align2::CENTER_CENTER,
            subtitle,
            egui::FontId::proportional(15.0),
            PREVIEW_TEXT,
        );
        draw_outlined_text(
            &painter,
            rect.left_top() + Vec2::new(18.0, 20.0),
            egui::Align2::LEFT_TOP,
            format!("{scene_count} scenes"),
            egui::FontId::proportional(13.0),
            PREVIEW_TEXT,
        );
    }

    fn draw_metadata_panel(&self, ui: &mut egui::Ui, summary: &ModSummary) {
        card_frame()
            .inner_margin(egui::Margin::same(18.0))
            .show(ui, |ui| {
                ui.set_min_width(320.0);
                ui.heading(RichText::new("Selected mod").size(16.0));
                ui.separator();
                metadata_row(ui, "id", &summary.id);
                metadata_row(ui, "version", &summary.version);
                metadata_row(ui, "status", summary.status.label());
                metadata_row(ui, "assets", &summary.asset_count.to_string());
                metadata_row(ui, "root", &summary.root.display().to_string());
                if let Some(detail) = summary.status.detail() {
                    ui.separator();
                    ui.colored_label(summary.status.color(), detail);
                }
                if !summary.capabilities.is_empty() {
                    ui.separator();
                    ui.heading(RichText::new("Capabilities").size(16.0));
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for capability in &summary.capabilities {
                            capability_chip(ui, capability);
                        }
                    });
                }
            });
    }

    fn draw_scenes_table(&mut self, ui: &mut egui::Ui, summary: &ModSummary) {
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("Scenes").size(16.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized([112.0, 28.0], egui::Button::new("Choose scene"));
                    });
                });
                ui.separator();
                let table_width = ui.available_width();
                let columns = scene_table_columns(table_width);
                draw_scene_header(ui, columns);

                egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    for (index, scene) in summary.scenes.iter().take(16).enumerate() {
                        let scene_selected = self.selected_scene.as_deref() == Some(scene.id.as_str());
                        let row_fill = if scene_selected {
                            Color32::from_rgb(235, 248, 246)
                        } else if index % 2 == 0 {
                            Color32::from_rgb(252, 253, 254)
                        } else {
                            Color32::from_rgb(247, 249, 250)
                        };
                        let row = egui::Frame::none()
                            .fill(row_fill)
                            .stroke(if scene_selected {
                                egui::Stroke::new(1.0, PRIMARY)
                            } else {
                                egui::Stroke::NONE
                            })
                            .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.set_width(table_width);
                                ui.horizontal(|ui| {
                                    let scene_cell = ui.add_sized(
                                        [columns[0], 28.0],
                                        egui::SelectableLabel::new(
                                            scene_selected,
                                            RichText::new(&scene.id).monospace(),
                                        ),
                                    )
                                    .on_hover_text(&scene.id);
                                    if scene_cell.clicked() {
                                        self.select_scene(&scene.id);
                                    }
                                    ui.add_sized([columns[1], 28.0], egui::Label::new(&scene.label))
                                        .on_hover_text(if scene.description.is_empty() {
                                            scene.label.as_str()
                                        } else {
                                            scene.description.as_str()
                                        });
                                    ui.add_sized(
                                        [columns[2], 28.0],
                                        egui::Label::new(if scene.launcher_visible { "Visible" } else { "Hidden" }),
                                    );
                                    ui.add_sized(
                                        [columns[3], 28.0],
                                        egui::Label::new(RichText::new("Ready").color(SUCCESS)),
                                    );
                                    if ui.add_sized([columns[4], 24.0], egui::Button::new("Open")).clicked() {
                                        self.select_scene(&scene.id);
                                        self.status = format!("Open scene requested: {}:{}", summary.id, scene.id);
                                    }
                                });
                            });
                        if row.response.clicked() {
                            self.select_scene(&scene.id);
                        }
                        ui.add_space(4.0);
                    }
                });
            });
        });
    }

    fn draw_start_dialog(&mut self, ctx: &egui::Context) {
        self.draw_navbar(ctx);
        self.draw_status_bar(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BODY_BG).inner_margin(egui::Margin::same(BODY_MARGIN)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let gap = BODY_MARGIN;
                let left_width = SIDEBAR_WIDTH
                    .min(available.x * 0.34)
                    .min((available.x - gap - 480.0).max(320.0))
                    .max(280.0);

                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(left_width, available.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.draw_dialog_mods_panel(ui);
                        },
                    );

                    ui.add_space(gap);

                    let right_width = ui.available_width();
                    ui.allocate_ui_with_layout(
                        Vec2::new(right_width, available.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.draw_dialog_selected_panel(ui);
                        },
                    );
                });
            });
    }

    fn draw_navbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("navbar")
            .exact_height(NAVBAR_HEIGHT)
            .frame(egui::Frame::none().fill(PRIMARY).inner_margin(egui::Margin::symmetric(24.0, 14.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Object Browse").color(Color32::WHITE).strong().size(20.0));
                    ui.label(RichText::new(APP_TITLE).color(Color32::from_rgb(220, 226, 232)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_sized([86.0, 30.0], egui::Button::new("Refresh")).clicked() {
                            self.refresh_mods();
                        }
                        ui.label(RichText::new("profile: dev").background_color(Color32::WHITE).color(PRIMARY));
                    });
                });
            });
    }

    fn draw_dialog_mods_panel(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Mods");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} found", self.mods.len())).small().color(TEXT_MUTED));
                });
            });
            ui.label(RichText::new(format!("found in {}", self.mods_root.display())).small().color(TEXT_MUTED));
            ui.add_space(8.0);
            ui.add_sized(
                [ui.available_width(), 28.0],
                egui::TextEdit::singleline(&mut self.search).hint_text("Search mods..."),
            );
            ui.horizontal(|ui| {
                let button_w = (ui.available_width() - 8.0) * 0.5;
                ui.add_sized([button_w, 28.0], egui::Button::new("All status"));
                ui.add_sized([button_w, 28.0], egui::Button::new("Name"));
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for summary in self.filtered_mods() {
                    self.draw_mod_card(ui, &summary);
                    ui.add_space(8.0);
                }
            });
        });
    }

    fn draw_dialog_selected_panel(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            let Some(summary) = self.selected().cloned() else {
                ui.centered_and_justified(|ui| {
                    ui.label("No mod selected");
                });
                return;
            };

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(&summary.name);
                    ui.label(RichText::new(&summary.description).color(TEXT_MUTED));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_sized([156.0, 34.0], egui::Button::new("Open default scene"));
                });
            });
            ui.separator();

            let content_width = ui.available_width();
            if content_width >= 880.0 {
                let gap = 24.0;
                let metadata_width = (content_width * 0.36).clamp(300.0, 420.0);
                let preview_width = (content_width - metadata_width - gap).max(360.0);
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(preview_width, 340.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| self.draw_preview_placeholder(ui, &summary, Vec2::new(preview_width, 320.0)),
                    );
                    ui.add_space(gap);
                    ui.allocate_ui_with_layout(
                        Vec2::new(metadata_width, 340.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.draw_preview_transport_controls(ui);
                            ui.separator();
                            self.draw_metadata_panel(ui, &summary);
                        },
                    );
                });
            } else {
                self.draw_preview_transport_controls(ui);
                self.draw_preview_placeholder(ui, &summary, Vec2::new(content_width, 300.0));
                ui.add_space(14.0);
                self.draw_metadata_panel(ui, &summary);
            }
            ui.add_space(18.0);
            self.draw_scenes_table(ui, &summary);
        });
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(STATUS_HEIGHT)
            .frame(egui::Frame::none().fill(BODY_BG).inner_margin(egui::Margin::symmetric(24.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("root: {}", self.mods_root.display()));
                    ui.separator();
                    ui.label(format!(
                        "preview mode: {:?} @ {} fps",
                        self.playback.label(),
                        PREVIEW_FPS
                    ));
                    ui.separator();
                    ui.label(&self.status);
                    let ready = self.mods.iter().filter(|m| matches!(m.status, ModStatus::Ready)).count();
                    let warnings = self.mods.iter().filter(|m| matches!(m.status, ModStatus::Warning(_))).count();
                    let errors = self.mods.iter().filter(|m| matches!(m.status, ModStatus::Error(_))).count();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Ready: {ready} | Warnings: {warnings} | Errors: {errors}"));
                    });
                });
            });
    }

    fn draw_rendered_snapshot(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        image: &SceneSnapshotImage,
    ) {
        let key = image.cache_key();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [image.width as usize, image.height as usize],
            &image.pixels_rgba8,
        );

        if let Some(cache) = self.preview_texture.as_mut() {
            if cache.width != image.width || cache.height != image.height {
                let texture_id = ui.ctx().load_texture(
                    format!("scene-snapshot:{key}"),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                cache.key = key;
                cache.width = image.width;
                cache.height = image.height;
                cache.handle = texture_id;
            } else {
                cache.key = key;
                cache.handle.set(color_image, egui::TextureOptions::LINEAR);
            }
            cache.width = image.width;
            cache.height = image.height;
        } else {
            let handle = ui.ctx().load_texture(
                format!("scene-snapshot:{key}"),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.preview_texture = Some(PreviewTextureCache {
                key,
                width: image.width,
                height: image.height,
                handle,
            });
        }

        let Some(cache) = &self.preview_texture else {
            draw_preview_error(ui, rect, "Snapshot texture upload failed");
            return;
        };

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, PREVIEW_BG);
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, Color32::from_rgb(55, 62, 76)));

        let image_rect = fit_rect(rect, image.width as f32 / image.height.max(1) as f32);
        painter.image(
            cache.handle.id(),
            image_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        draw_outlined_text(
            &painter,
            rect.left_top() + Vec2::new(18.0, 16.0),
            egui::Align2::LEFT_TOP,
            "Rendered Scene Snapshot",
            egui::FontId::proportional(15.0),
            PREVIEW_TEXT,
        );
        draw_outlined_text(
            &painter,
            rect.left_bottom() + Vec2::new(18.0, -18.0),
            egui::Align2::LEFT_BOTTOM,
            &image.diagnostic_label,
            egui::FontId::monospace(12.0),
            PREVIEW_TEXT,
        );
    }

    fn draw_preview_transport_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let selected_scene = self.selected_scene_for_current_mod().is_some();
            let pause_enabled = selected_scene && self.playback != PreviewPlayback::Pause;
            let play_enabled = selected_scene && self.playback != PreviewPlayback::Play;
            let stop_enabled = !matches!(self.playback, PreviewPlayback::Stop) || selected_scene;

            if ui
                .add_enabled(play_enabled, egui::Button::new("Play").fill(if self.playback == PreviewPlayback::Play { Color32::from_rgb(46, 204, 113) } else { Color32::from_rgb(240, 244, 248) }))
                .clicked()
            {
                self.set_playback(PreviewPlayback::Play);
            }
            if ui
                .add_enabled(pause_enabled, egui::Button::new("Pause").fill(if self.playback == PreviewPlayback::Pause { Color32::from_rgb(241, 196, 15) } else { Color32::from_rgb(240, 244, 248) }))
                .clicked()
            {
                self.set_playback(PreviewPlayback::Pause);
            }
            if ui
                .add_enabled(stop_enabled, egui::Button::new("Stop").fill(if self.playback == PreviewPlayback::Stop { Color32::from_rgb(231, 76, 60) } else { Color32::from_rgb(240, 244, 248) }))
                .clicked()
            {
                self.set_playback(PreviewPlayback::Stop);
            }
        });
    }
}

impl eframe::App for ObjectBrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);
        self.maybe_update_live_preview();
        if self.preview.tick() {
            ctx.request_repaint();
        }
        self.draw_start_dialog(ctx);
        if self.playback == PreviewPlayback::Play {
            ctx.request_repaint_after(PREVIEW_FRAME_INTERVAL);
        }
    }
}

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = BODY_BG;
    visuals.window_fill = CARD_BG;
    visuals.extreme_bg_color = BODY_BG;
    visuals.selection.bg_fill = Color32::from_rgb(233, 242, 247);
    visuals.selection.stroke.color = PRIMARY;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    ctx.set_visuals(visuals);
}

fn mods_root_for_mod(mod_root: &Path) -> PathBuf {
    mod_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("mods"))
}

fn metadata_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        let label_width = (ui.available_width() * 0.36).clamp(86.0, 148.0);
        ui.add_sized([label_width, 20.0], egui::Label::new(RichText::new(label).color(TEXT_MUTED)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
}

fn preview_scene_for(summary: &ModSummary) -> Option<&SceneSummary> {
    summary
        .scenes
        .iter()
        .find(|scene| scene.id.eq_ignore_ascii_case("menu"))
        .or_else(|| summary.scenes.iter().find(|scene| scene.launcher_visible))
        .or_else(|| summary.scenes.first())
}

fn status_badge(ui: &mut egui::Ui, label: &str, color: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 22.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 11.0, Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 28));
    painter.rect_stroke(rect, 11.0, egui::Stroke::new(1.0, color));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        color,
    );
}

fn draw_preview_loading_overlay(ui: &mut egui::Ui, rect: egui::Rect) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, PREVIEW_BG);
    painter.text(
        rect.center() - Vec2::new(0.0, 24.0),
        egui::Align2::CENTER_CENTER,
        "Loading preview...",
        egui::FontId::proportional(22.0),
        PREVIEW_TEXT,
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 8.0),
        egui::Align2::CENTER_CENTER,
        "Core Runtime is preparing the selected scene",
        egui::FontId::proportional(14.0),
        Color32::from_rgb(180, 184, 194),
    );

    let time = ui.input(|input| input.time) as f32;
    let spinner_center = rect.center() + Vec2::new(0.0, 54.0);
    for index in 0..8 {
        let angle = time * 5.0 + index as f32 / 8.0 * std::f32::consts::TAU;
        let pulse = ((time * 4.0 + index as f32 * 0.6).sin() + 1.0) * 0.5;
        let alpha = (80.0 + pulse * 150.0) as u8;
        let offset = Vec2::new(angle.cos() * 18.0, angle.sin() * 18.0);
        painter.circle_filled(
            spinner_center + offset,
            3.0,
            Color32::from_rgba_premultiplied(PRIMARY.r(), PRIMARY.g(), PRIMARY.b(), alpha),
        );
    }
}

fn draw_outlined_text(
    painter: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: impl ToString,
    font: egui::FontId,
    color: Color32,
) {
    let text = text.to_string();
    for offset in [
        Vec2::new(-1.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, -1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, 1.0),
    ] {
        painter.text(
            pos + offset,
            align,
            &text,
            font.clone(),
            PREVIEW_TEXT_OUTLINE,
        );
    }
    painter.text(pos, align, text, font, color);
}

fn draw_preview_error(ui: &mut egui::Ui, rect: egui::Rect, message: &str) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, Color32::from_rgb(255, 247, 247));
    painter.text(
        rect.center() - Vec2::new(0.0, 18.0),
        egui::Align2::CENTER_CENTER,
        "Preview error",
        egui::FontId::proportional(22.0),
        Color32::from_rgb(231, 76, 60),
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 18.0),
        egui::Align2::CENTER_CENTER,
        message,
        egui::FontId::proportional(14.0),
        TEXT_MUTED,
    );
}

fn draw_preview_snapshot(ui: &mut egui::Ui, rect: egui::Rect, snapshot: &PreviewSnapshot) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, PREVIEW_BG);
    painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, Color32::from_rgb(55, 62, 76)));

    painter.text(
        rect.left_top() + Vec2::new(18.0, 16.0),
        egui::Align2::LEFT_TOP,
        "Static Scene Preview",
        egui::FontId::proportional(15.0),
        TEXT_MUTED,
    );
    painter.text(
        rect.left_top() + Vec2::new(18.0, 38.0),
        egui::Align2::LEFT_TOP,
        format!("{} / {}", snapshot.info.mod_id, snapshot.info.scene_id),
        egui::FontId::monospace(13.0),
        PRIMARY,
    );
    painter.text(
        rect.right_top() + Vec2::new(-18.0, 18.0),
        egui::Align2::RIGHT_TOP,
        format!("{} entities", snapshot.entities_count),
        egui::FontId::proportional(13.0),
        TEXT_MUTED,
    );

    let viewport = egui::Rect::from_min_max(
        rect.min + Vec2::new(24.0, 70.0),
        rect.max - Vec2::new(24.0, 24.0),
    );
    painter.rect_filled(viewport, 4.0, Color32::from_rgb(14, 17, 22));
    painter.rect_stroke(viewport, 4.0, egui::Stroke::new(1.0, BORDER));

    let Some(bounds) = snapshot_bounds(snapshot) else {
        painter.text(
            viewport.center(),
            egui::Align2::CENTER_CENTER,
            &snapshot.info.scene_label,
            egui::FontId::proportional(26.0),
            PRIMARY,
        );
        return;
    };

    let bounds_w = (bounds.2 - bounds.0).max(1.0);
    let bounds_h = (bounds.3 - bounds.1).max(1.0);
    let scale = (viewport.width() / bounds_w)
        .min(viewport.height() / bounds_h)
        .min(3.0)
        * 0.82;
    let world_center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);

    for item in &snapshot.draw_items {
        draw_snapshot_item(&painter, viewport, world_center, scale, item);
    }
}

fn draw_snapshot_item(
    painter: &egui::Painter,
    viewport: egui::Rect,
    world_center: Vec2,
    scale: f32,
    item: &PreviewDrawItem,
) {
    match item {
        PreviewDrawItem::Rect {
            x,
            y,
            w,
            h,
            color,
            label,
        } => {
            let center = world_to_preview(viewport, world_center, scale, *x, *y);
            let size = Vec2::new((*w * scale).abs().max(6.0), (*h * scale).abs().max(6.0));
            let rect = egui::Rect::from_center_size(center, size);
            painter.rect_filled(rect, 4.0, preview_color(*color));
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, PRIMARY));
            if let Some(label) = label {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }
        PreviewDrawItem::Circle {
            x,
            y,
            r,
            color,
            label,
        } => {
            let center = world_to_preview(viewport, world_center, scale, *x, *y);
            let radius = (*r * scale).abs().clamp(4.0, 42.0);
            painter.circle_filled(center, radius, preview_color(*color));
            painter.circle_stroke(center, radius, egui::Stroke::new(1.0, PRIMARY));
            if let Some(label) = label {
                painter.text(
                    center + Vec2::new(0.0, radius + 10.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    PRIMARY,
                );
            }
        }
        PreviewDrawItem::Label { x, y, text, color } => {
            let pos = world_to_preview(viewport, world_center, scale, *x, *y);
            painter.text(
                pos,
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(18.0),
                preview_color(*color),
            );
        }
    }
}

fn world_to_preview(
    viewport: egui::Rect,
    world_center: Vec2,
    scale: f32,
    x: f32,
    y: f32,
) -> egui::Pos2 {
    let local = Vec2::new(x, y) - world_center;
    viewport.center() + Vec2::new(local.x * scale, -local.y * scale)
}

fn snapshot_bounds(snapshot: &PreviewSnapshot) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for item in &snapshot.draw_items {
        let (x0, y0, x1, y1) = match item {
            PreviewDrawItem::Rect { x, y, w, h, .. } => (
                *x - *w * 0.5,
                *y - *h * 0.5,
                *x + *w * 0.5,
                *y + *h * 0.5,
            ),
            PreviewDrawItem::Circle { x, y, r, .. } => (*x - *r, *y - *r, *x + *r, *y + *r),
            PreviewDrawItem::Label { x, y, .. } => (*x - 48.0, *y - 18.0, *x + 48.0, *y + 18.0),
        };
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
    }

    if min_x.is_finite() {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

fn preview_color(color: PreviewColor) -> Color32 {
    Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

fn fit_rect(container: egui::Rect, aspect: f32) -> egui::Rect {
    let aspect = aspect.max(0.01);
    let container_aspect = container.width() / container.height().max(1.0);
    let size = if container_aspect > aspect {
        Vec2::new(container.height() * aspect, container.height())
    } else {
        Vec2::new(container.width(), container.width() / aspect)
    };
    egui::Rect::from_center_size(container.center(), size)
}

fn capability_chip(ui: &mut egui::Ui, label: &str) {
    let width = (label.chars().count() as f32 * 7.0 + 18.0).clamp(72.0, 190.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 24.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 12.0, Color32::from_rgb(248, 249, 250));
    painter.rect_stroke(rect, 12.0, egui::Stroke::new(1.0, BORDER));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        PRIMARY,
    );
}

fn scene_table_columns(total_width: f32) -> [f32; 5] {
    let action = 76.0;
    let visibility = 92.0;
    let status = 92.0;
    let gutters = 8.0 * 4.0;
    let remaining = (total_width - action - visibility - status - gutters).max(260.0);
    let scene = (remaining * 0.42).clamp(140.0, 260.0);
    let label = (remaining - scene).max(160.0);
    [scene, label, visibility, status, action]
}

fn draw_scene_header(ui: &mut egui::Ui, columns: [f32; 5]) {
    egui::Frame::none()
        .fill(Color32::from_rgb(241, 245, 247))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized([columns[0], 22.0], egui::Label::new(RichText::new("Scene").strong()));
                ui.add_sized([columns[1], 22.0], egui::Label::new(RichText::new("Label").strong()));
                ui.add_sized([columns[2], 22.0], egui::Label::new(RichText::new("Visibility").strong()));
                ui.add_sized([columns[3], 22.0], egui::Label::new(RichText::new("Status").strong()));
                ui.add_sized([columns[4], 22.0], egui::Label::new(RichText::new("Action").strong()));
            });
        });
    ui.add_space(6.0);
}

fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(6.0)
        .inner_margin(egui::Margin::same(16.0))
}

fn scan_mods(mods_root: &Path) -> Vec<ModSummary> {
    let Ok(entries) = fs::read_dir(mods_root) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|entry| scan_single_mod(entry.path()))
        .collect()
}

fn scan_single_mod(root: PathBuf) -> ModSummary {
    let fallback_id = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    let manifest_path = root.join("mod.toml");
    let manifest_content = fs::read_to_string(&manifest_path);
    let manifest = manifest_content
        .as_deref()
        .ok()
        .and_then(|content| toml::from_str::<ModManifest>(content).ok());

    let status = match (&manifest_content, &manifest) {
        (Err(err), _) => ModStatus::Error(format!("Cannot read mod.toml: {err}")),
        (Ok(_), None) => ModStatus::Error(String::from("Cannot parse mod.toml")),
        (Ok(_), Some(manifest)) if manifest.scenes.as_ref().is_none_or(Vec::is_empty) => {
            ModStatus::Warning(String::from("No scenes declared"))
        }
        _ => ModStatus::Ready,
    };

    let scenes = manifest
        .as_ref()
        .and_then(|manifest| manifest.scenes.as_ref())
        .map(|scenes| {
            scenes
                .iter()
                .map(|scene| SceneSummary {
                    id: scene.id.clone(),
                    label: scene.label.clone().unwrap_or_else(|| scene.id.clone()),
                    description: scene.description.clone().unwrap_or_default(),
                    launcher_visible: scene.launcher_visible.unwrap_or(false),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let manifest_ref = manifest.as_ref();
    ModSummary {
        id: manifest_ref.map(|m| m.id.clone()).unwrap_or_else(|| fallback_id.clone()),
        name: manifest_ref
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| fallback_id.clone()),
        version: manifest_ref
            .and_then(|m| m.version.clone())
            .unwrap_or_else(|| String::from("unknown")),
        description: manifest_ref
            .and_then(|m| m.description.clone())
            .unwrap_or_else(|| String::from("No description")),
        root: root.clone(),
        status,
        categories: manifest_ref
            .and_then(|m| m.launcher_category.clone())
            .unwrap_or_default(),
        capabilities: manifest_ref
            .and_then(|m| m.capabilities.clone())
            .unwrap_or_default(),
        scenes,
        asset_count: count_content_files(&root),
    }
}

fn count_content_files(root: &Path) -> usize {
    ["assets", "audio", "fonts", "textures", "tilesets", "scripts", "scenes"]
        .iter()
        .map(|name| count_files_recursive(&root.join(name)))
        .sum()
}

fn count_files_recursive(path: &Path) -> usize {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                count_files_recursive(&entry.path())
            } else {
                1
            }
        })
        .sum()
}

fn parse_arg(args: &[String], name: &str) -> Option<String> {
    let key = format!("--{}", name);
    args.windows(2).find_map(|pair| {
        if pair[0] == key {
            Some(pair[1].clone())
        } else {
            None
        }
    })
}

fn main() -> eframe::Result {
    let args = std::env::args().collect::<Vec<_>>();
    let mods_root = parse_arg(&args, "mods-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("mods"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_TITLE)
            .with_inner_size([1440.0, 900.0]),
        ..Default::default()
    };

        eframe::run_native(
        APP_TITLE,
        options,
        Box::new(move |_cc| Ok(Box::new(ObjectBrowserApp::new(mods_root)))),
    )
}

impl PreviewWorker {
    fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<PreviewWorkerCommand>();
        let (result_tx, result_rx) = mpsc::channel::<PreviewWorkerResult>();

        let handle = std::thread::spawn(move || {
            let mut host: Option<ScenePreviewHost> = None;
            let mut active_token: u64 = 0;
            let mut active_key: Option<String> = None;
            let mut active_request: Option<SceneSnapshotRequest> = None;

            while let Ok(command) = command_rx.recv() {
                match command {
                    PreviewWorkerCommand::Configure {
                        token,
                        scene_key,
                        options,
                        snapshot_request,
                    } => {
                        host = Some(ScenePreviewHost::new(options));
                        active_token = token;
                        active_key = Some(scene_key);
                        active_request = Some(snapshot_request);
                    }
                    PreviewWorkerCommand::CaptureNextFrame { token, scene_key } => {
                        if token != active_token {
                            continue;
                        }
                        if active_key.as_deref() != Some(&scene_key) {
                            continue;
                        }

                        let Some(request) = active_request.clone() else {
                            let _ = result_tx.send(PreviewWorkerResult::RenderError {
                                token,
                                scene_key,
                                message: String::from("Missing live snapshot request"),
                            });
                            continue;
                        };

                        let result = match host.as_mut() {
                            Some(host) => host.capture_next_frame(),
                            None => {
                                let _ = result_tx.send(PreviewWorkerResult::RenderError {
                                    token,
                                    scene_key,
                                    message: String::from("Live preview host not configured"),
                                });
                                continue;
                            }
                        };

                        match result {
                            Ok(frame) => {
                                let mut snapshot_request = request;
                                snapshot_request.width = frame.width;
                                snapshot_request.height = frame.height;

                                let image = SceneSnapshotImage {
                                    request: snapshot_request,
                                    width: frame.width,
                                    height: frame.height,
                                    pixels_rgba8: frame.pixels_rgba8,
                                    diagnostic_label: frame.diagnostic_label,
                                };
                                let _ = result_tx.send(PreviewWorkerResult::Rendered {
                                    token,
                                    scene_key,
                                    image,
                                });
                            }
                            Err(err) => {
                                let _ = result_tx.send(PreviewWorkerResult::RenderError {
                                    token,
                                    scene_key,
                                    message: format!("Live preview capture error: {err}"),
                                });
                            }
                        }
                    }
                    PreviewWorkerCommand::Stop => {
                        host = None;
                        active_key = None;
                        active_request = None;
                        active_token = 0;
                    }
                    PreviewWorkerCommand::Shutdown => break,
                }
            }
        });

        Self {
            command_tx,
            result_rx,
            handle: Some(handle),
        }
    }

    fn configure(
        &self,
        token: u64,
        scene_key: String,
        options: ScenePreviewOptions,
        snapshot_request: SceneSnapshotRequest,
    ) {
        let _ = self.command_tx.send(PreviewWorkerCommand::Configure {
            token,
            scene_key,
            options,
            snapshot_request,
        });
    }

    fn capture_next_frame(&self, token: u64, scene_key: String) {
        let _ = self.command_tx.send(PreviewWorkerCommand::CaptureNextFrame {
            token,
            scene_key,
        });
    }

    fn pull_result(&self) -> Result<PreviewWorkerResult, TryRecvError> {
        self.result_rx.try_recv()
    }
}

impl Drop for PreviewWorker {
    fn drop(&mut self) {
        let _ = self.command_tx.send(PreviewWorkerCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
