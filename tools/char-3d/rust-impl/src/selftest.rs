use crate::{
    app::build_svg,
    assets::{BUILT_INS, ModelKind, project_path},
    export,
    mesh::Mesh,
    pipeline::compute_frame,
    state::AppState,
};
use std::{fs, path::PathBuf};

pub fn run(output_dir: Option<PathBuf>) -> anyhow::Result<()> {
    let root = output_dir.unwrap_or_else(|| std::env::temp_dir().join("char-3d-rust-self-test"));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;

    let mut atlas_frames = Vec::new();
    for model in BUILT_INS {
        let mesh = match model.kind {
            ModelKind::Obj => Mesh::from_obj_file(&project_path(model.path), model.label)?,
            ModelKind::Fbx => Mesh::from_fbx_file(&project_path(model.path), model.label)?,
            ModelKind::AnimClip => {
                Mesh::from_anim_clip_file(&project_path(model.path), model.label)?
            }
        };
        let mut state = AppState {
            model_source: model.id.to_owned(),
            ..Default::default()
        };
        if matches!(model.kind, ModelKind::Fbx | ModelKind::AnimClip) {
            state.reset_view_for_fbx();
            state.auto = true;
            let duration = mesh.animation_duration().unwrap_or(1.35);
            state.advance_animation(1.0 / state.anim_fps.max(1.0), duration);
        } else {
            state.reset_view_for_obj();
        }
        let frame = compute_frame(&mesh, &state, 640, 420);
        anyhow::ensure!(
            frame.stats.screen_faces > 0,
            "model produced no screen faces: {}",
            model.id
        );
        let stem = model.id.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        let png_path = root.join(format!("{stem}.png"));
        let svg_path = root.join(format!("{stem}.svg"));
        export::save_png(&frame, &png_path)?;
        fs::write(&svg_path, build_svg(&frame))?;
        ensure_non_empty(&png_path)?;
        ensure_non_empty(&svg_path)?;
        atlas_frames.push((model.id, frame));
    }

    export::save_atlas(&atlas_frames, &root.join("all_models_atlas.png"))?;
    ensure_non_empty(&root.join("all_models_atlas.png"))?;
    println!("self-test ok: {}", root.display());
    Ok(())
}

fn ensure_non_empty(path: &std::path::Path) -> anyhow::Result<()> {
    let len = fs::metadata(path)?.len();
    anyhow::ensure!(len > 0, "empty output file: {}", path.display());
    Ok(())
}
