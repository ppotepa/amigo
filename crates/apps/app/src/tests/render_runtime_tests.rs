use super::*;
use amigo_2d_composition::RenderLayer2dSceneService;
use amigo_layered_image_2d_plugin::{
    LayeredImageBlendMode2d, LayeredImageDrawCommand, LayeredImageInstance,
    LayeredImageSceneService, LayeredImageViewportFit2d,
};
use amigo_light_2d_plugin::{GlobalLight2dSceneService, LightGroup2dSceneService};

#[test]
#[ignore = "requires a local WGPU adapter for offscreen readback"]
fn playground_npr_preview_renders_paper_and_ink_edges() {
    let frame = crate::capture_scene_preview(
        crate::ScenePreviewOptions::new(mods_root(), "playground-npr", "comic-lines", 320, 240)
            .with_active_mods(vec!["core".to_owned(), "playground-npr".to_owned()])
            .with_warmup_frames(2),
    )
    .expect("npr scene preview should render offscreen");

    assert_npr_preview_has_paper_and_ink(&frame, "gpu_realtime default_gpu_comic", 800, 80);
}

#[test]
#[ignore = "requires a local WGPU adapter for offscreen readback"]
fn playground_npr_preview_renders_gpu_and_cpu_reference_default_gpu_comic() {
    let mut host = crate::ScenePreviewHost::new(
        crate::ScenePreviewOptions::new(mods_root(), "playground-npr", "comic-lines", 320, 240)
            .with_active_mods(vec!["core".to_owned(), "playground-npr".to_owned()])
            .with_warmup_frames(2),
    );
    let gpu_frame = host
        .capture_rgba8()
        .expect("gpu npr scene preview should render offscreen");
    assert_npr_preview_has_paper_and_ink(&gpu_frame, "gpu_realtime default_gpu_comic", 800, 80);

    host.apply_mesh3d_npr_preset(
        "playground-npr-model-1-soldier",
        "default_gpu_comic_cpu_reference",
    )
    .expect("cpu reference npr preset should apply");
    host.warmup(1)
        .expect("cpu reference npr scene preview should advance");
    let cpu_frame = host
        .capture_rgba8()
        .expect("cpu reference npr scene preview should render offscreen");
    assert_npr_preview_has_paper_and_ink(&cpu_frame, "cpu_reference default_gpu_comic", 200, 120);
    assert_npr_ink_masks_are_similar(&gpu_frame, &cpu_frame, "default_gpu_comic");
}

fn assert_npr_preview_has_paper_and_ink(
    frame: &crate::ScenePreviewFrame,
    label: &str,
    min_dark_pixels: usize,
    min_adjacent_ink_pixels: usize,
) {
    let bright_pixels = count_bright_pixels(&frame.pixels_rgba8);
    let dark_pixels = count_dark_pixels(&frame.pixels_rgba8);
    let nonwhite_pixels = count_nonwhite_pixels(&frame.pixels_rgba8);
    let min_luma = min_pixel_luma(&frame.pixels_rgba8);
    let ink_edge_pixels = count_dark_pixels_adjacent_to_bright(
        &frame.pixels_rgba8,
        frame.width as usize,
        frame.height as usize,
    );

    assert!(
        bright_pixels > 200,
        "{label} NPR preview should contain visible paper/background pixels, got {bright_pixels}"
    );
    assert!(
        dark_pixels > min_dark_pixels,
        "{label} NPR preview should contain a visible ink drawing, got dark={dark_pixels}, nonwhite={nonwhite_pixels}, bright={bright_pixels}, min_luma={min_luma}, adjacent_ink={ink_edge_pixels}"
    );
    assert!(
        ink_edge_pixels > min_adjacent_ink_pixels,
        "{label} NPR preview should contain dark ink pixels adjacent to model fill, got {ink_edge_pixels}; dark={dark_pixels}, nonwhite={nonwhite_pixels}, bright={bright_pixels}, min_luma={min_luma}"
    );
}

fn count_bright_pixels(pixels: &[u8]) -> usize {
    pixels
        .chunks_exact(4)
        .filter(|rgba| pixel_luma(rgba) > 150)
        .count()
}

fn assert_npr_ink_masks_are_similar(
    gpu_frame: &crate::ScenePreviewFrame,
    cpu_frame: &crate::ScenePreviewFrame,
    label: &str,
) {
    assert_eq!(gpu_frame.width, cpu_frame.width);
    assert_eq!(gpu_frame.height, cpu_frame.height);
    let gpu_mask = ink_edge_mask(
        &gpu_frame.pixels_rgba8,
        gpu_frame.width as usize,
        gpu_frame.height as usize,
    );
    let cpu_mask = ink_edge_mask(
        &cpu_frame.pixels_rgba8,
        cpu_frame.width as usize,
        cpu_frame.height as usize,
    );
    let gpu_stats = mask_stats(&gpu_mask, gpu_frame.width as usize, gpu_frame.height as usize);
    let cpu_stats = mask_stats(&cpu_mask, cpu_frame.width as usize, cpu_frame.height as usize);
    let count_ratio = if cpu_stats.count == 0 {
        0.0
    } else {
        gpu_stats.count as f32 / cpu_stats.count as f32
    };
    let centroid_dx = (gpu_stats.centroid_x - cpu_stats.centroid_x).abs();
    let centroid_dy = (gpu_stats.centroid_y - cpu_stats.centroid_y).abs();
    let bbox_intersects = gpu_stats.count > 0
        && cpu_stats.count > 0
        && gpu_stats.min_x <= cpu_stats.max_x
        && gpu_stats.max_x >= cpu_stats.min_x
        && gpu_stats.min_y <= cpu_stats.max_y
        && gpu_stats.max_y >= cpu_stats.min_y;
    assert!(
        bbox_intersects
            && (0.30..=3.00).contains(&count_ratio)
            && centroid_dx <= gpu_frame.width as f32 * 0.14
            && centroid_dy <= gpu_frame.height as f32 * 0.18,
        "{label} GPU/CPU NPR ink masks should occupy the same model region for A/B parity smoke test, got count_ratio={count_ratio:.3}, centroid_dx={centroid_dx:.2}, centroid_dy={centroid_dy:.2}, bbox_intersects={bbox_intersects}, gpu={gpu_stats:?}, cpu={cpu_stats:?}"
    );
}

#[derive(Debug)]
struct MaskStats {
    count: usize,
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
    centroid_x: f32,
    centroid_y: f32,
}

fn mask_stats(mask: &[bool], width: usize, height: usize) -> MaskStats {
    let mut count = 0usize;
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    let mut sum_x = 0usize;
    let mut sum_y = 0usize;

    for y in 0..height {
        for x in 0..width {
            if !mask[y * width + x] {
                continue;
            }
            count += 1;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            sum_x += x;
            sum_y += y;
        }
    }

    if count == 0 {
        return MaskStats {
            count,
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
            centroid_x: 0.0,
            centroid_y: 0.0,
        };
    }

    MaskStats {
        count,
        min_x,
        min_y,
        max_x,
        max_y,
        centroid_x: sum_x as f32 / count as f32,
        centroid_y: sum_y as f32 / count as f32,
    }
}

fn ink_edge_mask(pixels: &[u8], width: usize, height: usize) -> Vec<bool> {
    let mut mask = vec![false; width.saturating_mul(height)];
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let index = (y * width + x) * 4;
            let pixel = &pixels[index..index + 4];
            if pixel_luma(pixel) > 100 {
                continue;
            }
            let has_bright_neighbor = [
                (x - 1, y),
                (x + 1, y),
                (x, y - 1),
                (x, y + 1),
                (x - 1, y - 1),
                (x + 1, y - 1),
                (x - 1, y + 1),
                (x + 1, y + 1),
            ]
            .into_iter()
            .any(|(nx, ny)| {
                let neighbor = (ny * width + nx) * 4;
                pixel_luma(&pixels[neighbor..neighbor + 4]) > 150
            });
            if has_bright_neighbor {
                mask[y * width + x] = true;
            }
        }
    }
    mask
}

fn count_dark_pixels_adjacent_to_bright(pixels: &[u8], width: usize, height: usize) -> usize {
    let mut count = 0;
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let index = (y * width + x) * 4;
            let pixel = &pixels[index..index + 4];
            if pixel_luma(pixel) > 100 {
                continue;
            }
            let has_bright_neighbor = [
                (x - 1, y),
                (x + 1, y),
                (x, y - 1),
                (x, y + 1),
                (x - 1, y - 1),
                (x + 1, y - 1),
                (x - 1, y + 1),
                (x + 1, y + 1),
            ]
            .into_iter()
            .any(|(nx, ny)| {
                let neighbor = (ny * width + nx) * 4;
                pixel_luma(&pixels[neighbor..neighbor + 4]) > 150
            });
            if has_bright_neighbor {
                count += 1;
            }
        }
    }
    count
}

fn count_dark_pixels(pixels: &[u8]) -> usize {
    pixels
        .chunks_exact(4)
        .filter(|rgba| pixel_luma(rgba) <= 100)
        .count()
}

fn count_nonwhite_pixels(pixels: &[u8]) -> usize {
    pixels
        .chunks_exact(4)
        .filter(|rgba| rgba[0] < 245 || rgba[1] < 245 || rgba[2] < 245)
        .count()
}

fn min_pixel_luma(pixels: &[u8]) -> u8 {
    pixels
        .chunks_exact(4)
        .map(pixel_luma)
        .min()
        .unwrap_or(255)
}

fn pixel_luma(rgba: &[u8]) -> u8 {
    ((rgba[0] as u16 + rgba[1] as u16 + rgba[2] as u16) / 3) as u8
}

#[test]
fn handle_script_command_asset_reload_requests_load_and_event() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-sidescroller".to_owned()),
        Some("vertical-slice".to_owned()),
        Vec::new(),
        true,
    );
    asset_catalog.register_manifest(AssetManifest {
        key: AssetKey::new("playground-sidescroller/audio/jump"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        tags: vec!["audio".to_owned(), "generated".to_owned()],
    });

    script_runtime::dispatch_script_command(
        ScriptCommand::new(
            "asset",
            "reload",
            vec!["playground-sidescroller/audio/jump".to_owned()],
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert!(
        asset_catalog
            .pending_loads()
            .iter()
            .any(|request| request.key.as_str() == "playground-sidescroller/audio/jump")
    );
    assert!(script_event_queue.pending().iter().any(|event| {
        event.topic == "asset.reload-requested"
            && event.payload == vec!["playground-sidescroller/audio/jump".to_owned()]
    }));
}

#[test]
fn handle_script_command_queues_and_processes_audio_state() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let audio_state = AudioStateService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-sidescroller".to_owned()),
        Some("vertical-slice".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::audio_play("jump"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::audio_start_realtime("proximity-beep"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::audio_set_param("proximity-beep", "distance", 128.0),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    let commands = audio_command_queue.drain();
    assert_eq!(commands.len(), 3);
    assert_eq!(audio_scene_service.clips().len(), 2);

    for command in commands {
        process_audio_command(command, &audio_state, &dev_console_state);
    }

    assert!(audio_state.playing_sources().contains_key("proximity-beep"));
    assert_eq!(audio_state.drain_runtime_commands().len(), 3);
    assert_eq!(
        audio_state
            .source_params()
            .get("proximity-beep")
            .and_then(|params| params.get("distance"))
            .copied(),
        Some(128.0)
    );
}

#[test]
fn handle_script_command_queues_scene_commands() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("screen-space-preview".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "select", vec!["sprite-showcase".to_owned()]),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "reload", Vec::new()),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "spawn", vec!["runtime-test-entity".to_owned()]),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "clear", Vec::new()),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    let commands = scene_command_queue.pending();
    assert!(matches!(
        commands.first(),
        Some(SceneCommand::SelectScene { scene }) if scene.as_str() == "sprite-showcase"
    ));
    assert!(matches!(
        commands.get(1),
        Some(SceneCommand::ReloadActiveScene)
    ));
    assert!(matches!(
        commands.get(2),
        Some(SceneCommand::SpawnNamedEntity { name, transform }) if name == "runtime-test-entity" && transform.is_none()
    ));
    assert!(matches!(commands.get(3), Some(SceneCommand::ClearEntities)));
}

#[test]
fn handle_script_command_unknown_command_reports_fallback() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("screen-space-preview".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::new("unknown", "noop", vec!["x".to_owned()]),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert!(
        dev_console_state
            .output_lines()
            .iter()
            .any(|line| line.contains("unhandled placeholder script command: unknown.noop(x)"))
    );
}

#[test]
fn handle_script_command_updates_ui_state() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("screen-space-preview".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::ui_set_text("playground-2d-ui-preview.subtitle", "Updated from Rhai"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_set_value("playground-2d-ui-preview.hp-bar", 0.5),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_hide("playground-2d-ui-preview.root"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_disable(
            "playground-2d-ui-preview.root.control-card.button-row.repair-button",
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_enable(
            "playground-2d-ui-preview.root.control-card.button-row.repair-button",
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert_eq!(
        ui_state
            .text_override("playground-2d-ui-preview.subtitle")
            .as_deref(),
        Some("Updated from Rhai")
    );
    assert_eq!(
        ui_state.value_override("playground-2d-ui-preview.hp-bar"),
        Some(0.5)
    );
    assert!(!ui_state.is_visible("playground-2d-ui-preview.root"));
    assert!(
        ui_state.is_enabled("playground-2d-ui-preview.root.control-card.button-row.repair-button")
    );
}

#[test]
fn handle_script_command_updates_layered_image_overrides() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let layered_images = LayeredImageSceneService::default();
    let render_layers = RenderLayer2dSceneService::default();
    let global_lights = GlobalLight2dSceneService::default();
    let light_groups = LightGroup2dSceneService::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("test-mod".to_owned()),
        Some("main-menu".to_owned()),
        Vec::new(),
        true,
    );
    layered_images.queue(LayeredImageDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(1),
        entity_name: "test-layered-background".to_owned(),
        image: LayeredImageInstance {
            asset: AssetKey::new("test-mod/layered-images/test-pack"),
            size: amigo_math::Vec2::new(1280.0, 720.0),
            base_opacity: 0.0,
            viewport_fit: LayeredImageViewportFit2d::Fixed,
            layer_overrides: Vec::new(),
            visual_maps: None,
        },
        render_layer: "default".to_owned(),
        z_index: -100.0,
        transform: amigo_math::Transform2::default(),
    });

    for command in [
        ScriptCommand::new(
            "2d.layered_image",
            "set_base_opacity",
            vec!["test-layered-background".to_owned(), "0.35".to_owned()],
        ),
        ScriptCommand::new(
            "2d.layered_image",
            "set_opacity",
            vec![
                "test-layered-background".to_owned(),
                "accent_light".to_owned(),
                "0.42".to_owned(),
            ],
        ),
        ScriptCommand::new(
            "2d.layered_image",
            "set_enabled",
            vec![
                "test-layered-background".to_owned(),
                "accent_light".to_owned(),
                "false".to_owned(),
            ],
        ),
        ScriptCommand::new(
            "2d.layered_image",
            "set_blend",
            vec![
                "test-layered-background".to_owned(),
                "accent_light".to_owned(),
                "screen".to_owned(),
            ],
        ),
    ] {
        script_runtime::dispatch_script_command_with_layered_image_service(
            command,
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &layered_images,
            &render_layers,
            &global_lights,
            &light_groups,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
    }

    let command = layered_images.commands().remove(0);
    assert_eq!(command.image.base_opacity, 0.35);
    let override_ = command
        .image
        .layer_overrides
        .iter()
        .find(|override_| override_.id == "accent_light")
        .expect("script command should create accent light override");
    assert_eq!(override_.opacity, Some(0.42));
    assert_eq!(override_.enabled, Some(false));
    assert_eq!(override_.blend_mode, Some(LayeredImageBlendMode2d::Screen));

    script_runtime::dispatch_script_command_with_layered_image_service(
        ScriptCommand::new(
            "2d.layered_image",
            "set_blend",
            vec![
                "test-layered-background".to_owned(),
                "accent_light".to_owned(),
                "overlay".to_owned(),
            ],
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &layered_images,
        &render_layers,
        &global_lights,
        &light_groups,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert!(
        dev_console_state
            .output_lines()
            .iter()
            .any(|line| { line.contains("invalid layered image blend mode `overlay`") })
    );
}

#[test]
fn handle_script_command_writes_debug_text_export() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(None, None, Vec::new(), true);
    let relative_path = format!(
        "tests/debug-export-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );
    let target_path = PathBuf::from("target")
        .join("amigo-dev-exports")
        .join(&relative_path);
    if target_path.exists() {
        fs::remove_file(&target_path).expect("stale debug export should be removable");
    }

    script_runtime::dispatch_script_command(
        ScriptCommand::new(
            "debug",
            "write-text",
            vec![relative_path.clone(), "hello export".to_owned()],
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert_eq!(
        fs::read_to_string(&target_path).expect("debug export should be written"),
        "hello export"
    );
}

#[test]
fn resolve_existing_asset_path_prefers_metadata_candidates() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("amigo-asset-path-{unique}"));
    fs::create_dir_all(root.join("textures")).expect("temp textures dir should exist");

    let metadata_path = root.join("textures").join("player.sprite.yml");
    fs::create_dir_all(
        metadata_path
            .parent()
            .expect("metadata parent should exist"),
    )
    .expect("metadata parent should be created");
    fs::write(&metadata_path, "kind: sprite-sheet-2d\nimage: player.png\n")
        .expect("metadata file should be created");

    let resolved = crate::assets::resolve_existing_asset_path(
        root.join("textures").join("player"),
        "test/player",
    )
    .expect("metadata candidate should resolve");

    assert_eq!(resolved, metadata_path);
}
