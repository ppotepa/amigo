use std::fs;
use std::path::Path;
use std::path::PathBuf;

use amigo_modding::DiscoveredMod;

use crate::cache::inputs::{collect_preview_inputs, relative_input_paths};
use crate::cache::project_id::project_cache_id_for_root;
use crate::dto::{DiagnosticLevel, EditorDiagnosticDto, PreviewStatus, ScenePreviewDto};
use crate::mods::discovery::default_mods_root;
use crate::preview::cache::scene_preview_cache_dir;
use crate::preview::hashing::source_hash;
use crate::preview::slideshow::{
    STARTUP_PREVIEW_FPS, STARTUP_PREVIEW_FRAME_COUNT, STARTUP_PREVIEW_HEIGHT,
    STARTUP_PREVIEW_SECONDS, STARTUP_PREVIEW_WIDTH, cached_engine_slideshow,
    generate_engine_slideshow,
};

pub fn request_scene_preview(
    discovered: &DiscoveredMod,
    scene_id: &str,
    force_regenerate: bool,
    cache_root: &Path,
    on_frame_generated: impl Fn(u32, u32),
) -> Result<ScenePreviewDto, String> {
    let scene = discovered.scene_by_id(scene_id).ok_or_else(|| {
        format!(
            "scene `{scene_id}` was not found in mod `{}`",
            discovered.manifest.id
        )
    })?;
    let document_path = scene.document_path(&discovered.root_path);
    let preview_inputs = collect_preview_inputs(&discovered.root_path, scene);
    let preview_input_refs = preview_inputs.iter().map(PathBuf::as_path).collect::<Vec<_>>();
    let preview_input_labels = relative_input_paths(&discovered.root_path, &preview_inputs);
    let hash = source_hash(
        &preview_input_refs,
        STARTUP_PREVIEW_WIDTH,
        STARTUP_PREVIEW_HEIGHT,
        STARTUP_PREVIEW_FPS,
        STARTUP_PREVIEW_FRAME_COUNT,
    );

    if !document_path.is_file() {
        return Ok(ScenePreviewDto {
            mod_id: discovered.manifest.id.clone(),
            scene_id: scene_id.to_owned(),
            status: PreviewStatus::Missing,
            fps: STARTUP_PREVIEW_FPS,
            frame_count: 0,
            image_url: None,
            frame_urls: Vec::new(),
            width: STARTUP_PREVIEW_WIDTH,
            height: STARTUP_PREVIEW_HEIGHT,
            duration_ms: 0,
            generated_at: None,
            source_hash: hash,
            diagnostics: vec![EditorDiagnosticDto {
                level: DiagnosticLevel::Error,
                code: "missing_scene_document".to_owned(),
                message: format!("Scene document `{}` is missing.", document_path.display()),
                path: Some(document_path.display().to_string()),
            }],
        });
    }

    let project_cache_id = project_cache_id_for_root(&discovered.root_path);
    let output_dir = scene_preview_cache_dir(cache_root, &project_cache_id, scene_id, &hash);
    let generated = if force_regenerate {
        None
    } else {
        cached_engine_slideshow(&discovered.manifest.id, scene_id, &hash, &output_dir)
    }
    .map(Ok)
    .unwrap_or_else(|| {
        generate_engine_slideshow(
            &default_mods_root(),
            &project_cache_id,
            &discovered.manifest.id,
            scene_id,
            scene.label.as_str(),
            hash.as_str(),
            &preview_input_labels,
            active_mods_for_preview(discovered),
            &output_dir,
            on_frame_generated,
        )
    });

    let generated = match generated {
        Ok(generated) => generated,
        Err(error) => {
            return Ok(ScenePreviewDto {
                mod_id: discovered.manifest.id.clone(),
                scene_id: scene_id.to_owned(),
                status: PreviewStatus::Failed,
                fps: STARTUP_PREVIEW_FPS,
                frame_count: 0,
                image_url: None,
                frame_urls: Vec::new(),
                width: STARTUP_PREVIEW_WIDTH,
                height: STARTUP_PREVIEW_HEIGHT,
                duration_ms: 0,
                generated_at: Some(current_unix_seconds()),
                source_hash: hash,
                diagnostics: vec![EditorDiagnosticDto {
                    level: DiagnosticLevel::Error,
                    code: "engine_preview_failed".to_owned(),
                    message: format!("Engine preview generation failed: {error}"),
                    path: Some(document_path.display().to_string()),
                }],
            });
        }
    };

    let frame_urls = generated
        .frame_paths
        .iter()
        .map(|path| frame_url(path))
        .collect::<Result<Vec<_>, _>>()?;

    let mut diagnostics = generated.diagnostics;
    if force_regenerate && !generated.cache_hit {
        diagnostics.push(EditorDiagnosticDto {
            level: DiagnosticLevel::Info,
            code: "preview_cache_refreshed".to_owned(),
            message: "Regenerated preview cache for selected scene.".to_owned(),
            path: Some(output_dir.display().to_string()),
        });
    }

    Ok(ScenePreviewDto {
        mod_id: discovered.manifest.id.clone(),
        scene_id: scene_id.to_owned(),
        status: PreviewStatus::Ready,
        fps: generated.fps,
        frame_count: generated.frame_count,
        image_url: frame_urls.first().cloned(),
        frame_urls,
        width: generated.width,
        height: generated.height,
        duration_ms: STARTUP_PREVIEW_SECONDS * 1000,
        generated_at: Some(current_unix_seconds()),
        source_hash: hash,
        diagnostics,
    })
}

fn active_mods_for_preview(discovered: &DiscoveredMod) -> Vec<String> {
    let mut active_mods = discovered.manifest.dependencies.clone();
    if !active_mods.contains(&discovered.manifest.id) {
        active_mods.push(discovered.manifest.id.clone());
    }
    active_mods
}

fn frame_url(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read preview frame `{}`: {error}", path.display()))?;
    Ok(format!("data:image/png;base64,{}", encode_base64(&bytes)))
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let value = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;

        encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);

        if chunk.len() > 1 {
            encoded.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }

        if chunk.len() > 2 {
            encoded.push(TABLE[(value & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}

fn current_unix_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}
