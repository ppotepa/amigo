//! External egui host for engine-declared panels.
use amigo_panel_api::*;
use amigo_runtime_control::ControlValue;
use amigo_scene::{SceneUiNodeComponentDocument as Node, SceneUiNodeTypeComponentDocument as Kind};
use eframe::egui;
use std::{collections::BTreeMap, sync::mpsc, time::Duration};

pub fn run() -> Result<(), String> {
    let (tx, incoming) = mpsc::sync_channel(128);
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        loop {
            match read_message::<ServerMessage>(&mut stdin) {
                Ok(message) => {
                    if tx.send(message).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    let _ = tx.send(ServerMessage::Shutdown);
                    break;
                }
            }
        }
    });
    let (outgoing, rx) = mpsc::sync_channel(128);
    let writer = std::thread::spawn(move || {
        let mut stdout = std::io::stdout().lock();
        while let Ok(message) = rx.recv() {
            if write_message(&mut stdout, &message).is_err() {
                break;
            }
        }
    });
    outgoing
        .send(ClientMessage::Hello {
            version: PROTOCOL_VERSION,
        })
        .map_err(|e| e.to_string())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 860.0])
            .with_min_inner_size([840.0, 640.0])
            // A scene panel must remain discoverable when monitors are added,
            // removed, or rearranged. Persisting an old physical position can
            // otherwise launch a healthy process entirely off-screen.
            .with_position([24.0, 24.0]),
        persist_window: false,
        ..Default::default()
    };
    let close_sender = outgoing.clone();
    let result = eframe::run_native(
        "Amigo scene panel",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
                style.spacing.item_spacing = egui::vec2(10.0, 9.0);
                style.spacing.button_padding = egui::vec2(10.0, 6.0);
                style.spacing.slider_width = 165.0;
                style.visuals.selection.bg_fill = egui::Color32::from_rgb(48, 103, 128);
            });
            Ok(Box::new(PanelApp {
                incoming,
                outgoing,
                document: None,
                values: BTreeMap::new(),
                pending: BTreeMap::new(),
                preset_names: Vec::new(),
                asset_browser: None,
                selected_asset_source: None,
                selected_asset: None,
                asset_query: String::new(),
                asset_browser_expanded: true,
                confirm: None,
                generation: 0,
                revision: 0,
                request: 0,
                error: None,
                tabs: BTreeMap::new(),
            }))
        }),
    )
    .map_err(|e| e.to_string());
    if result.is_ok() {
        let _ = close_sender.send(ClientMessage::Close);
    }
    drop(close_sender);
    let _ = writer.join();
    result
}
struct PanelApp {
    incoming: mpsc::Receiver<ServerMessage>,
    outgoing: mpsc::SyncSender<ClientMessage>,
    document: Option<PanelDocument>,
    values: BTreeMap<String, PropertySnapshot>,
    pending: BTreeMap<String, (u64, ControlValue)>,
    preset_names: Vec<String>,
    asset_browser: Option<amigo_panel_api::PanelAssetBrowserSnapshot>,
    selected_asset_source: Option<String>,
    selected_asset: Option<String>,
    asset_query: String,
    asset_browser_expanded: bool,
    confirm: Option<String>,
    generation: u64,
    revision: u64,
    request: u64,
    error: Option<String>,
    tabs: BTreeMap<String, String>,
}
impl PanelApp {
    fn send(&mut self, message: ClientMessage) -> bool {
        if let Err(e) = self.outgoing.try_send(message) {
            self.error = Some(format!("connection busy/disconnected: {e}"));
            return false;
        }
        true
    }
    fn send_asset_intent(&mut self, usage: String) {
        let Some(document) = self.document.as_ref() else {
            return;
        };
        let Some(browser) = self.asset_browser.as_ref() else {
            return;
        };
        let Some(source_id) = self.selected_asset_source.as_ref() else {
            return;
        };
        let Some(source) = browser
            .sources
            .iter()
            .find(|source| &source.id == source_id)
        else {
            return;
        };
        let Some(asset_id) = self
            .selected_asset
            .as_ref()
            .filter(|id| source.entries.iter().any(|entry| &entry.id == *id))
        else {
            return;
        };
        self.request += 1;
        self.send(ClientMessage::AssetIntent {
            request: self.request,
            intent: amigo_panel_api::PanelAssetIntent {
                panel_id: document.id.clone(),
                generation: self.generation,
                revision: self.revision,
                source_id: source.id.clone(),
                source_revision: source.revision,
                asset_id: asset_id.clone(),
                usage,
            },
        });
    }
    fn refresh_selected_asset_source(&mut self) {
        let Some(panel_id) = self.document.as_ref().map(|document| document.id.clone()) else {
            return;
        };
        let Some(browser) = self.asset_browser.as_ref() else {
            return;
        };
        let Some(source_id) = self.selected_asset_source.as_ref() else {
            return;
        };
        let Some(source) = browser
            .sources
            .iter()
            .find(|source| &source.id == source_id)
        else {
            return;
        };
        self.request += 1;
        self.send(ClientMessage::RefreshAssetSource {
            request: self.request,
            panel_id,
            generation: self.generation,
            revision: self.revision,
            source_id: source.id.clone(),
            source_revision: source.revision,
        });
    }
    fn asset_browser(&mut self, ui: &mut egui::Ui) {
        let Some(browser) = self.asset_browser.clone() else {
            return;
        };
        let source_id = self
            .selected_asset_source
            .clone()
            .filter(|id| browser.sources.iter().any(|source| source.id == *id))
            .or(browser.selected_source.clone())
            .or_else(|| browser.sources.first().map(|source| source.id.clone()));
        let Some(source_id) = source_id else {
            return;
        };
        self.selected_asset_source = Some(source_id.clone());
        let Some(source) = browser.sources.iter().find(|source| source.id == source_id) else {
            return;
        };
        let selected_source_before = self.selected_asset_source.clone();
        ui.small("Choose a source, then use the selected asset in the scene.");
        ui.separator();
        {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("asset-source")
                    .selected_text(&source.label)
                    .show_ui(ui, |ui| {
                        for choice in &browser.sources {
                            ui.selectable_value(
                                &mut self.selected_asset_source,
                                Some(choice.id.clone()),
                                &choice.label,
                            );
                        }
                    });
                match source.state {
                    amigo_panel_api::PanelAssetSourceState::Empty => {
                        ui.label("No assets");
                    }
                    amigo_panel_api::PanelAssetSourceState::Loading => {
                        ui.label("Loading…");
                    }
                    amigo_panel_api::PanelAssetSourceState::Ready => {
                        ui.label("Ready");
                    }
                    amigo_panel_api::PanelAssetSourceState::Failed { failed_assets } => {
                        ui.colored_label(
                            egui::Color32::LIGHT_RED,
                            format!("{failed_assets} failed"),
                        );
                    }
                }
                if source.refreshable && ui.small_button("Refresh").clicked() {
                    self.refresh_selected_asset_source();
                }
            });
            ui.text_edit_singleline(&mut self.asset_query)
                .on_hover_text("Filter assets in this source");
            let query = self.asset_query.to_lowercase();
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for entry in source.entries.iter().filter(|entry| {
                        query.is_empty()
                            || entry.label.to_lowercase().contains(&query)
                            || entry
                                .tags
                                .iter()
                                .any(|tag| tag.to_lowercase().contains(&query))
                    }) {
                        let selected = self.selected_asset.as_deref() == Some(&entry.id);
                        let glyph = if entry.kind.is_some() { "◇" } else { "·" };
                        if ui
                            .add_sized(
                                [ui.available_width(), 36.0],
                                egui::Button::selectable(
                                    selected,
                                    format!("{glyph}  {}", entry.label),
                                ),
                            )
                            .clicked()
                        {
                            self.selected_asset = Some(entry.id.clone());
                        }
                        ui.small(format!("{} · {:?}", entry.tags.join(", "), entry.state));
                    }
                });
            let selected_ready = self.selected_asset.as_ref().is_some_and(|id| {
                source.entries.iter().any(|entry| {
                    entry.id == *id
                        && matches!(entry.state, amigo_panel_api::PanelAssetEntryState::Ready)
                })
            });
            let actions = self
                .document
                .as_ref()
                .and_then(|document| document.asset_browser.clone());
            if let Some(actions) = actions {
                ui.horizontal(|ui| {
                    if let Some(usage) = actions.add_usage {
                        if ui
                            .add_enabled(selected_ready, egui::Button::new("Add to scene"))
                            .clicked()
                        {
                            self.send_asset_intent(usage);
                        }
                    }
                    if let Some(usage) = actions.replace_usage {
                        if ui
                            .add_enabled(selected_ready, egui::Button::new("Replace selected"))
                            .clicked()
                        {
                            self.send_asset_intent(usage);
                        }
                    }
                });
            }
        }
        if self.selected_asset_source != selected_source_before {
            self.selected_asset = None;
        }
    }
    fn options_for(&self, node: &Node) -> Vec<String> {
        node.options_bind
            .as_ref()
            .and_then(|path| self.values.get(path))
            .and_then(|value| value.value.as_string())
            .map(|value| {
                value
                    .split('\n')
                    .filter(|option| !option.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| node.options.clone())
    }
    fn node(&mut self, ui: &mut egui::Ui, node: &Node) {
        if node
            .visible_bind
            .as_ref()
            .and_then(|p| self.values.get(p))
            .is_some_and(|p| p.value == ControlValue::Bool(false))
        {
            return;
        }
        let enabled = !node
            .enabled_bind
            .as_ref()
            .and_then(|p| self.values.get(p))
            .is_some_and(|p| p.value == ControlValue::Bool(false));
        let id = node.id.as_deref().unwrap_or("layout");
        let hint = self
            .document
            .as_ref()
            .and_then(|d| d.presentation.get(id))
            .cloned()
            .unwrap_or_default();
        ui.push_id(id, |ui| {
            ui.add_enabled_ui(enabled, |ui| match node.kind {
                Kind::Row => {
                    ui.horizontal_wrapped(|ui| {
                        for child in &node.children {
                            self.node(ui, child);
                        }
                    });
                }
                Kind::Panel | Kind::Column | Kind::Stack => {
                    ui.vertical(|ui| {
                        for child in &node.children {
                            self.node(ui, child);
                        }
                    });
                }
                Kind::GroupBox => {
                    egui::CollapsingHeader::new(node.text.as_deref().unwrap_or(id))
                        .default_open(!hint.collapsed)
                        .show(ui, |ui| {
                            for child in &node.children {
                                self.node(ui, child);
                            }
                        });
                }
                Kind::TabView => {
                    let selected = self.tabs.entry(id.to_owned()).or_insert_with(|| {
                        node.tabs.first().map(|t| t.id.clone()).unwrap_or_default()
                    });
                    if !node.tabs.iter().any(|t| t.id == *selected) {
                        *selected = node.tabs.first().map(|t| t.id.clone()).unwrap_or_default();
                    }
                    let columns = ((ui.available_width() / 112.0).floor() as usize).max(1);
                    for row in node.tabs.chunks(columns) {
                        ui.horizontal_wrapped(|ui| {
                            for tab in row {
                                let active = *selected == tab.id;
                                let label = if active {
                                    egui::RichText::new(&tab.label).strong()
                                } else {
                                    egui::RichText::new(&tab.label)
                                };
                                if ui
                                    .add_sized(
                                        [104.0, 30.0],
                                        egui::Button::new(label).selected(active),
                                    )
                                    .clicked()
                                {
                                    *selected = tab.id.clone();
                                }
                            }
                        });
                    }
                    let selected = selected.clone();
                    let selected_label = node
                        .tabs
                        .iter()
                        .find(|tab| tab.id == selected)
                        .map(|tab| tab.label.as_str())
                        .unwrap_or("Workspace");
                    ui.separator();
                    ui.label(egui::RichText::new(selected_label).heading().strong());
                    for child in &node.children {
                        if child.id.as_deref() == Some(&selected) {
                            egui::ScrollArea::vertical()
                                .id_salt((id, &selected))
                                .show(ui, |ui| self.node(ui, child));
                        }
                    }
                }
                Kind::Text => {
                    let value = node.text_bind.as_ref().and_then(|p| self.values.get(p));
                    ui.label(
                        value
                            .map(|p| {
                                format!(
                                    "{}: {}",
                                    node.text.as_deref().unwrap_or(id),
                                    display(&p.value)
                                )
                            })
                            .unwrap_or_else(|| node.text.clone().unwrap_or_default()),
                    );
                }
                Kind::Button => {
                    if ui.button(node.text.as_deref().unwrap_or(id)).clicked() {
                        if self
                            .document
                            .as_ref()
                            .is_some_and(|d| d.confirm_actions.iter().any(|action| action == id))
                        {
                            self.confirm = Some(id.to_owned());
                        } else {
                            self.click(id);
                        }
                    }
                }
                Kind::Spacer => {
                    ui.add_space(node.style.height.unwrap_or(8.0));
                }
                _ => self.property(ui, node),
            })
        });
    }
    fn click(&mut self, id: &str) {
        self.request += 1;
        self.send(ClientMessage::Click {
            request: self.request,
            generation: self.generation,
            revision: self.revision,
            control: id.to_owned(),
        });
    }
    fn property(&mut self, ui: &mut egui::Ui, node: &Node) {
        let id = node.id.as_deref().unwrap_or("property");
        let Some(path) = &node.value_bind else {
            ui.colored_label(egui::Color32::YELLOW, format!("{id}: missing value_bind"));
            return;
        };
        let Some(snapshot) = self.values.get(path).cloned() else {
            ui.label(format!("{id}: waiting for {path}"));
            return;
        };
        let mut value = snapshot.value.clone();
        let label = node.text.as_deref().unwrap_or(id);
        let hint = self
            .document
            .as_ref()
            .and_then(|d| d.presentation.get(id))
            .cloned()
            .unwrap_or_default();
        let options = self.options_for(node);
        let changed = ui
            .add_enabled_ui(snapshot.writable, |ui| {
                if !matches!(value, ControlValue::Bool(_)) {
                    ui.horizontal(|ui| {
                        ui.label(label).on_hover_text(
                            hint.tooltip
                                .as_deref()
                                .or(snapshot.description.as_deref())
                                .unwrap_or(label),
                        );
                        if snapshot.writable
                            && hint.reset != Some(false)
                            && hint.choices.is_empty()
                            && ui
                                .small_button("Reset")
                                .on_hover_text("Restore the authored default")
                                .clicked()
                        {
                            self.request += 1;
                            self.send(ClientMessage::Reset {
                                request: self.request,
                                generation: self.generation,
                                revision: self.revision,
                                control: id.into(),
                            });
                            self.pending.remove(path);
                        }
                    });
                }
                match &mut value {
                    ControlValue::Bool(v) => ui
                        .checkbox(v, label)
                        .on_hover_text(hint.tooltip.as_deref().unwrap_or(label))
                        .changed(),
                    ControlValue::F64(v) => {
                        let range = snapshot.range.as_ref();
                        let min = node.min.map(f64::from).or(range.and_then(|r| r.min));
                        let max = node.max.map(f64::from).or(range.and_then(|r| r.max));
                        match (min, max) {
                            (Some(a), Some(b)) => ui
                                .add(
                                    egui::Slider::new(v, a..=b)
                                        .suffix(hint.suffix.as_deref().unwrap_or(""))
                                        .step_by(node.step.unwrap_or(0.01) as f64),
                                )
                                .changed(),
                            _ => ui
                                .add(egui::DragValue::new(v).speed(node.step.unwrap_or(0.01)))
                                .changed(),
                        }
                    }
                    ControlValue::I64(v) => ui.add(egui::DragValue::new(v)).changed(),
                    ControlValue::U64(v) => ui.add(egui::DragValue::new(v)).changed(),
                    ControlValue::String(v) | ControlValue::AssetRef(v) => {
                        if self
                            .document
                            .as_ref()
                            .and_then(|d| d.preset_name_bind.as_ref())
                            == Some(path)
                        {
                            let before = v.clone();
                            ui.text_edit_singleline(v);
                            egui::ComboBox::from_id_salt("saved-presets")
                                .selected_text("Saved presets")
                                .show_ui(ui, |ui| {
                                    for name in &self.preset_names {
                                        ui.selectable_value(v, name.clone(), name);
                                    }
                                });
                            *v != before
                        } else if !hint.choices.is_empty() && options == node.options {
                            let before = v.clone();
                            egui::ScrollArea::horizontal()
                                .id_salt("thumbnails")
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        for choice in &hint.choices {
                                            ui.vertical(|ui| {
                                                let (rect, response) = ui.allocate_exact_size(
                                                    egui::vec2(70.0, 70.0),
                                                    egui::Sense::click(),
                                                );
                                                ui.painter().rect_filled(
                                                    rect,
                                                    6.0,
                                                    if *v == choice.value {
                                                        ui.visuals().selection.bg_fill
                                                    } else {
                                                        egui::Color32::from_rgb(38, 42, 48)
                                                    },
                                                );
                                                let artwork = choice
                                                    .artwork_bind
                                                    .as_ref()
                                                    .and_then(|p| self.values.get(p))
                                                    .and_then(|p| p.value.as_string())
                                                    .or(choice.artwork.as_deref());
                                                if let Some(triangles) = artwork.and_then(|key| {
                                                    self.document.as_ref()?.artwork.get(key)
                                                }) {
                                                    let area = rect.shrink(4.0);
                                                    let mut mesh = egui::Mesh::default();
                                                    for triangle in triangles {
                                                        let base = mesh.vertices.len() as u32;
                                                        for p in &triangle.points {
                                                            mesh.colored_vertex(
                                                                area.min
                                                                    + egui::vec2(
                                                                        p[0] * area.width(),
                                                                        p[1] * area.height(),
                                                                    ),
                                                                egui::Color32::from_rgb(
                                                                    triangle.color[0],
                                                                    triangle.color[1],
                                                                    triangle.color[2],
                                                                ),
                                                            );
                                                        }
                                                        mesh.add_triangle(base, base + 1, base + 2);
                                                    }
                                                    // A single mesh avoids AA fringes on tiny/degenerate imported faces.
                                                    ui.painter()
                                                        .with_clip_rect(area)
                                                        .add(egui::Shape::mesh(mesh));
                                                }
                                                if response.on_hover_text(&choice.label).clicked() {
                                                    *v = choice.value.clone();
                                                }
                                                if ui
                                                    .selectable_label(
                                                        *v == choice.value,
                                                        &choice.label,
                                                    )
                                                    .clicked()
                                                {
                                                    *v = choice.value.clone();
                                                }
                                                if let Some(status) = choice
                                                    .status_bind
                                                    .as_ref()
                                                    .and_then(|p| self.values.get(p))
                                                {
                                                    ui.small(display(&status.value));
                                                }
                                            });
                                        }
                                    });
                                });
                            if hint.navigation {
                                let index = options.iter().position(|o| o == v).unwrap_or(0);
                                ui.horizontal(|ui| {
                                    if ui.button("< Previous").clicked() {
                                        *v = options[(index + options.len() - 1) % options.len()]
                                            .clone();
                                    }
                                    ui.label(format!(
                                        "{} / {} · {}",
                                        index + 1,
                                        options.len(),
                                        hint.choices
                                            .iter()
                                            .find(|c| c.value == *v)
                                            .map(|c| c.label.as_str())
                                            .unwrap_or(v)
                                    ));
                                    if ui.button("Next >").clicked() {
                                        *v = options[(index + 1) % options.len()].clone();
                                    }
                                });
                            }
                            *v != before
                        } else if options.is_empty()
                            && node.options_bind.is_some()
                            && matches!(node.kind, Kind::OptionSet | Kind::Dropdown)
                        {
                            ui.weak("No scene instances — add a model from Asset Browser.");
                            false
                        } else if options.is_empty() {
                            ui.text_edit_singleline(v).changed()
                        } else if node.kind == Kind::OptionSet {
                            let before = v.clone();
                            ui.horizontal_wrapped(|ui| {
                                for option in &options {
                                    ui.selectable_value(v, option.clone(), option);
                                }
                            });
                            *v != before
                        } else {
                            let before = v.clone();
                            egui::ComboBox::from_id_salt("choice")
                                .selected_text(v.as_str())
                                .show_ui(ui, |ui| {
                                    for option in &options {
                                        ui.selectable_value(v, option.clone(), option);
                                    }
                                });
                            *v != before
                        }
                    }
                    ControlValue::Color(v) => ui.color_edit_button_rgba_unmultiplied(v).changed(),
                    ControlValue::Vec2(v) => {
                        ui.horizontal(|ui| {
                            v.iter_mut().enumerate().fold(false, |changed, (i, x)| {
                                ui.add(
                                    egui::DragValue::new(x)
                                        .prefix(["X ", "Y "][i])
                                        .speed(node.step.unwrap_or(0.01))
                                        .suffix(hint.suffix.as_deref().unwrap_or("")),
                                )
                                .changed()
                                    || changed
                            })
                        })
                        .inner
                    }
                    ControlValue::Vec3(v) => {
                        ui.horizontal(|ui| {
                            v.iter_mut().enumerate().fold(false, |changed, (i, x)| {
                                ui.add(
                                    egui::DragValue::new(x)
                                        .prefix(["X ", "Y ", "Z "][i])
                                        .speed(node.step.unwrap_or(0.01))
                                        .suffix(hint.suffix.as_deref().unwrap_or("")),
                                )
                                .changed()
                                    || changed
                            })
                        })
                        .inner
                    }
                    ControlValue::Null => {
                        ui.label("unavailable");
                        false
                    }
                }
            })
            .inner;
        if changed {
            self.request += 1;
            if !self.send(ClientMessage::Edit {
                request: self.request,
                generation: self.generation,
                revision: self.revision,
                control: id.to_owned(),
                value: value.clone(),
            }) {
                return;
            }
            self.pending
                .insert(path.clone(), (self.request, value.clone()));
            if let Some(p) = self.values.get_mut(path) {
                p.value = value;
            }
        }
    }
}
fn display(value: &ControlValue) -> String {
    match value {
        ControlValue::String(v) | ControlValue::AssetRef(v) => v.clone(),
        ControlValue::F64(v) => format!("{v:.2}"),
        ControlValue::U64(v) => v.to_string(),
        ControlValue::I64(v) => v.to_string(),
        ControlValue::Bool(v) => if *v { "Yes" } else { "No" }.into(),
        v => format!("{v:?}"),
    }
}
impl eframe::App for PanelApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        while let Ok(message) = self.incoming.try_recv() {
            match message {
                ServerMessage::Document {
                    version,
                    generation,
                    revision,
                    document,
                } => {
                    if version != PROTOCOL_VERSION {
                        self.error = Some("unsupported panel protocol".into());
                        continue;
                    }
                    self.generation = generation;
                    self.revision = revision;
                    self.values.clear();
                    self.pending.clear();
                    self.confirm = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(document.title.clone()));
                    self.document = Some(document);
                    self.error = None;
                }
                ServerMessage::Snapshot {
                    generation,
                    revision,
                    acknowledged,
                    preset_names,
                    mut values,
                    asset_browser,
                } => {
                    if generation == self.generation && revision == self.revision {
                        self.preset_names = preset_names;
                        self.pending
                            .retain(|_, (request, _)| *request > acknowledged);
                        for (path, (_, value)) in &self.pending {
                            if let Some(property) = values.get_mut(path) {
                                property.value = value.clone();
                            }
                        }
                        self.values = values;
                        self.asset_browser = asset_browser;
                        if self.selected_asset_source.is_none() {
                            self.selected_asset_source = self
                                .asset_browser
                                .as_ref()
                                .and_then(|browser| browser.selected_source.clone());
                        }
                    }
                }
                ServerMessage::Result { request, error } => {
                    if error.is_some() {
                        self.pending.retain(|_, (r, _)| *r != request);
                    }
                    self.error = error;
                }
                ServerMessage::Diagnostic(error) => {
                    self.error = (!error.is_empty()).then_some(error)
                }
                ServerMessage::Shutdown => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
        if let Some(action) = self.confirm.clone() {
            egui::Window::new("Potwierdzenie")
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    let label = self
                        .document
                        .as_ref()
                        .and_then(|d| {
                            d.nodes()
                                .into_iter()
                                .find(|n| n.id.as_ref() == Some(&action))
                        })
                        .and_then(|n| n.text.as_deref())
                        .unwrap_or(&action);
                    ui.label(format!("Wykonać: {label}?"));
                    ui.horizontal(|ui| {
                        if ui.button("Potwierdź").clicked() {
                            self.click(&action);
                            self.confirm = None;
                        }
                        if ui.button("Anuluj").clicked() {
                            self.confirm = None;
                        }
                    });
                });
        }
        if self.asset_browser.is_some() {
            if self.asset_browser_expanded {
                egui::Panel::left("asset-browser")
                    .default_size(270.0)
                    .min_size(210.0)
                    .max_size(360.0)
                    .resizable(true)
                    .show(root, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Asset Browser");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("Hide").clicked() {
                                        self.asset_browser_expanded = false;
                                    }
                                },
                            );
                        });
                        self.asset_browser(ui);
                    });
            } else {
                egui::Panel::left("asset-browser-toggle")
                    .default_size(42.0)
                    .min_size(32.0)
                    .max_size(60.0)
                    .resizable(false)
                    .show(root, |ui| {
                        if ui.button("Assets ›").clicked() {
                            self.asset_browser_expanded = true;
                        }
                    });
            }
        }
        egui::CentralPanel::default().show(root, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
                ui.separator();
            }
            if let Some(doc) = self.document.clone() {
                ui.heading(&doc.title);
                if doc.presentation.values().all(|p| p.pin.is_none()) {
                    egui::ScrollArea::vertical().show(ui, |ui| self.node(ui, &doc.root));
                    return;
                }
                let pin = |node: &Node| {
                    node.id
                        .as_ref()
                        .and_then(|id| doc.presentation.get(id))
                        .and_then(|p| p.pin)
                };
                for node in &doc.root.children {
                    if pin(node) == Some(PanelPin::Top) {
                        self.node(ui, node);
                    }
                }
                ui.separator();
                // Bottom-up footer stays visible; the remaining body owns scrolling.
                let footer_height = if doc
                    .root
                    .children
                    .iter()
                    .any(|n| pin(n) == Some(PanelPin::Bottom))
                {
                    42.0
                } else {
                    0.0
                };
                let body_height = (ui.available_height() - footer_height).max(80.0);
                ui.allocate_ui(egui::vec2(ui.available_width(), body_height), |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if doc.root.children.is_empty() {
                                self.node(ui, &doc.root);
                            } else {
                                for node in &doc.root.children {
                                    if pin(node).is_none() {
                                        self.node(ui, node);
                                    }
                                }
                            }
                        });
                });
                for node in &doc.root.children {
                    if pin(node) == Some(PanelPin::Bottom) {
                        self.node(ui, node);
                    }
                }
            } else {
                ui.label("Connecting to scene…");
            }
        });
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}
