use std::fs;
use std::path::{Path, PathBuf};

use amigo_app::{ScenePreviewHost, ScenePreviewOptions};
use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

pub const STARTUP_PREVIEW_WIDTH: u32 = 640;
pub const STARTUP_PREVIEW_HEIGHT: u32 = 360;
pub const STARTUP_PREVIEW_FPS: u32 = 5;
pub const STARTUP_PREVIEW_SECONDS: u32 = 3;
pub const STARTUP_PREVIEW_FRAME_COUNT: u32 = STARTUP_PREVIEW_FPS * STARTUP_PREVIEW_SECONDS;

#[derive(Debug, Clone)]
pub struct GeneratedSlideshow {
    pub frame_paths: Vec<PathBuf>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frame_count: u32,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub cache_hit: bool,
}

pub const PREVIEW_RENDERER_VERSION: &str = "engine-slideshow-preview-v1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreviewManifest {
    kind: String,
    version: u32,
    project_cache_id: String,
    mod_id_at_generation: String,
    scene_label_at_generation: String,
    source_hash: String,
    renderer_version: String,
    mod_id: String,
    scene_id: String,
    width: u32,
    height: u32,
    fps: u32,
    frame_count: u32,
    generated_at: String,
    inputs: Vec<String>,
    frames: Vec<String>,
}

pub fn cached_engine_slideshow(
    mod_id: &str,
    scene_id: &str,
    source_hash: &str,
    output_dir: &Path,
) -> Option<GeneratedSlideshow> {
    let manifest = read_manifest(output_dir)?;
    if manifest.kind != "scene-preview-slideshow"
        || manifest.version != 1
        || manifest.renderer_version != PREVIEW_RENDERER_VERSION
        || manifest.source_hash != source_hash
        || manifest.width != STARTUP_PREVIEW_WIDTH
        || manifest.height != STARTUP_PREVIEW_HEIGHT
        || manifest.fps != STARTUP_PREVIEW_FPS
        || manifest.frame_count != STARTUP_PREVIEW_FRAME_COUNT
    {
        return None;
    }

    let frame_paths = manifest
        .frames
        .iter()
        .map(|frame| output_dir.join(frame))
        .collect::<Vec<_>>();

    if frame_paths.len() != STARTUP_PREVIEW_FRAME_COUNT as usize
        || !frame_paths.iter().all(|path| path.is_file())
    {
        return None;
    }

    Some(GeneratedSlideshow {
        frame_paths,
        width: STARTUP_PREVIEW_WIDTH,
        height: STARTUP_PREVIEW_HEIGHT,
        fps: STARTUP_PREVIEW_FPS,
        frame_count: STARTUP_PREVIEW_FRAME_COUNT,
        diagnostics: vec![EditorDiagnosticDto {
            level: DiagnosticLevel::Info,
            code: "engine_slideshow_preview_cache_hit".to_owned(),
            message: format!("Loaded cached engine slideshow preview for `{mod_id}/{scene_id}`."),
            path: Some(output_dir.display().to_string()),
        }],
        cache_hit: true,
    })
}

pub fn generate_engine_slideshow(
    mods_root: &Path,
    project_cache_id: &str,
    mod_id: &str,
    scene_id: &str,
    scene_label: &str,
    source_hash: &str,
    inputs: &[String],
    active_mods: Vec<String>,
    output_dir: &Path,
    on_frame_generated: impl Fn(u32, u32),
) -> Result<GeneratedSlideshow, String> {
    crate::preview::cache::ensure_cache_dir(output_dir)?;

    let options = ScenePreviewOptions::new(
        mods_root,
        mod_id,
        scene_id,
        STARTUP_PREVIEW_WIDTH,
        STARTUP_PREVIEW_HEIGHT,
    )
    .with_active_mods(active_mods)
    .with_warmup_frames(3)
    .with_playback_delta_seconds(1.0 / STARTUP_PREVIEW_FPS as f32);

    let mut host = ScenePreviewHost::new(options);
    let mut frame_paths = Vec::new();

    for frame_index in 0..STARTUP_PREVIEW_FRAME_COUNT {
        let frame = if frame_index == 0 {
            host.capture_rgba8()
        } else {
            host.capture_next_frame()
        }
        .map_err(|error| format!("failed to capture preview frame {frame_index}: {error}"))?;

        let frame_path = output_dir.join(format!("frame_{frame_index:03}.png"));
        save_rgba_png(&frame_path, frame.width, frame.height, &frame.pixels_rgba8)?;
        frame_paths.push(frame_path);
        on_frame_generated(frame_index + 1, STARTUP_PREVIEW_FRAME_COUNT);
    }

    write_manifest(
        output_dir,
        project_cache_id,
        mod_id,
        scene_id,
        scene_label,
        source_hash,
        inputs,
        &frame_paths,
    )?;

    Ok(GeneratedSlideshow {
        frame_paths,
        width: STARTUP_PREVIEW_WIDTH,
        height: STARTUP_PREVIEW_HEIGHT,
        fps: STARTUP_PREVIEW_FPS,
        frame_count: STARTUP_PREVIEW_FRAME_COUNT,
        diagnostics: vec![EditorDiagnosticDto {
            level: DiagnosticLevel::Info,
            code: "engine_slideshow_preview".to_owned(),
            message: "Generated real engine slideshow preview.".to_owned(),
            path: Some(output_dir.display().to_string()),
        }],
        cache_hit: false,
    })
}

fn write_manifest(
    output_dir: &Path,
    project_cache_id: &str,
    mod_id: &str,
    scene_id: &str,
    scene_label: &str,
    source_hash: &str,
    inputs: &[String],
    frame_paths: &[PathBuf],
) -> Result<(), String> {
    let manifest = PreviewManifest {
        kind: "scene-preview-slideshow".to_owned(),
        version: 1,
        project_cache_id: project_cache_id.to_owned(),
        mod_id_at_generation: mod_id.to_owned(),
        scene_label_at_generation: scene_label.to_owned(),
        source_hash: source_hash.to_owned(),
        renderer_version: PREVIEW_RENDERER_VERSION.to_owned(),
        mod_id: mod_id.to_owned(),
        scene_id: scene_id.to_owned(),
        width: STARTUP_PREVIEW_WIDTH,
        height: STARTUP_PREVIEW_HEIGHT,
        fps: STARTUP_PREVIEW_FPS,
        frame_count: STARTUP_PREVIEW_FRAME_COUNT,
        generated_at: unix_seconds(),
        inputs: inputs.to_vec(),
        frames: frame_paths
            .iter()
            .filter_map(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .collect(),
    };
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("failed to serialize preview manifest: {error}"))?;
    fs::write(output_dir.join("preview.json"), json)
        .map_err(|error| format!("failed to write preview manifest: {error}"))
}

fn read_manifest(output_dir: &Path) -> Option<PreviewManifest> {
    let text = fs::read_to_string(output_dir.join("preview.json")).ok()?;
    serde_json::from_str(&text).ok()
}

fn unix_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn save_rgba_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    let expected_len = width as usize * height as usize * 4;
    if pixels.len() != expected_len {
        return Err(format!(
            "invalid RGBA frame buffer size: expected {expected_len}, got {}",
            pixels.len()
        ));
    }

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| "failed to build RGBA image buffer".to_owned())?;

    image
        .save(path)
        .map_err(|error| format!("failed to save preview frame `{}`: {error}", path.display()))
}
