use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    AssetCatalog, AssetKey, AssetManifest, AssetSourceKind, LoadedAsset, PreparedAsset,
    PreparedAssetKind, prepare_asset_from_contents,
};

pub const DISCOVERED_MESH_3D_TAG: &str = "discovered-mesh-3d";
pub const MODEL_LIBRARY_MESH_3D_TAG: &str = "model-library-mesh-3d";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredMesh3dAsset {
    pub key: AssetKey,
    pub label: String,
    pub path: PathBuf,
}

pub fn discover_glb_mesh3d_assets(
    catalog: &AssetCatalog,
    mod_root: &Path,
    mod_id: &str,
) -> Result<Vec<DiscoveredMesh3dAsset>, String> {
    let (mut output, authored_source_paths) =
        discover_authored_mesh3d_assets(catalog, mod_root, mod_id)?;
    let source_root = mod_root.join("source-models");
    if !source_root.exists() {
        return Ok(output);
    }

    let mut paths = Vec::new();
    collect_glb_paths(&source_root, &mut paths)?;
    paths.sort();

    for path in paths {
        if canonical_asset_path(&path)
            .is_some_and(|canonical| authored_source_paths.contains(&canonical))
        {
            continue;
        }
        let Some(relative) = path.strip_prefix(mod_root).ok() else {
            continue;
        };
        let asset_key = AssetKey::new(format!(
            "{mod_id}/discovered-models/{}",
            discovered_key_fragment(relative)
        ));
        let label = discovered_label(&path);
        let byte_len = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let mut metadata = BTreeMap::new();
        metadata.insert("kind".to_owned(), "mesh-3d".to_owned());
        metadata.insert("format".to_owned(), "glb".to_owned());
        metadata.insert("label".to_owned(), label.clone());
        metadata.insert("source.file".to_owned(), path.display().to_string());
        metadata.insert(
            "source.discovered_from".to_owned(),
            source_root.display().to_string(),
        );
        enrich_mesh_animation_metadata(&mut metadata, &path);

        catalog.register_manifest(AssetManifest {
            key: asset_key.clone(),
            source: AssetSourceKind::Mod(mod_id.to_owned()),
            tags: vec![
                "mesh-3d".to_owned(),
                "model-3d".to_owned(),
                MODEL_LIBRARY_MESH_3D_TAG.to_owned(),
                DISCOVERED_MESH_3D_TAG.to_owned(),
            ],
        });
        catalog.mark_loaded(LoadedAsset {
            key: asset_key.clone(),
            source: AssetSourceKind::Mod(mod_id.to_owned()),
            resolved_path: path.clone(),
            byte_len,
        });
        catalog.mark_prepared(PreparedAsset {
            key: asset_key.clone(),
            source: AssetSourceKind::Mod(mod_id.to_owned()),
            resolved_path: path.clone(),
            byte_len,
            kind: PreparedAssetKind::Mesh3d,
            label: Some(label.clone()),
            format: Some("glb".to_owned()),
            metadata,
        });
        output.push(DiscoveredMesh3dAsset {
            key: asset_key,
            label,
            path,
        });
    }

    Ok(output)
}

pub fn discovered_mesh3d_assets(catalog: &AssetCatalog) -> Vec<DiscoveredMesh3dAsset> {
    let mut assets = catalog
        .prepared_assets()
        .into_iter()
        .filter(|asset| {
            matches!(asset.kind, PreparedAssetKind::Mesh3d)
                && catalog
                    .tags_for(&asset.key)
                    .iter()
                    .any(|tag| tag == MODEL_LIBRARY_MESH_3D_TAG || tag == DISCOVERED_MESH_3D_TAG)
        })
        .map(|asset| DiscoveredMesh3dAsset {
            key: asset.key,
            label: asset
                .label
                .or_else(|| asset.metadata.get("label").cloned())
                .unwrap_or_default(),
            path: asset.resolved_path,
        })
        .collect::<Vec<_>>();
    assets.sort_by(|left, right| left.label.cmp(&right.label).then(left.key.cmp(&right.key)));
    assets
}

fn discover_authored_mesh3d_assets(
    catalog: &AssetCatalog,
    mod_root: &Path,
    mod_id: &str,
) -> Result<(Vec<DiscoveredMesh3dAsset>, BTreeSet<PathBuf>), String> {
    let mesh_root = mod_root.join("meshes");
    if !mesh_root.exists() {
        return Ok((Vec::new(), BTreeSet::new()));
    }

    let mut paths = Vec::new();
    collect_yaml_paths(&mesh_root, &mut paths)?;
    paths.sort();

    let mut output = Vec::new();
    let mut source_paths = BTreeSet::new();
    for path in paths {
        let Some(relative) = path.strip_prefix(mod_root).ok() else {
            continue;
        };
        let asset_key = AssetKey::new(format!("{mod_id}/{}", discovered_key_fragment(relative)));
        catalog.register_manifest(AssetManifest {
            key: asset_key.clone(),
            source: AssetSourceKind::Mod(mod_id.to_owned()),
            tags: vec![
                "mesh-3d".to_owned(),
                "model-3d".to_owned(),
                MODEL_LIBRARY_MESH_3D_TAG.to_owned(),
            ],
        });

        let byte_len = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let loaded = LoadedAsset {
            key: asset_key.clone(),
            source: AssetSourceKind::Mod(mod_id.to_owned()),
            resolved_path: path.clone(),
            byte_len,
        };
        catalog.mark_loaded(loaded.clone());

        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) => {
                catalog.mark_failed(
                    asset_key,
                    format!("failed to read mesh asset `{}`: {error}", path.display()),
                );
                continue;
            }
        };
        let mut prepared = match prepare_asset_from_contents(&loaded, &contents) {
            Ok(prepared) => prepared,
            Err(error) => {
                catalog.mark_failed(asset_key, error);
                continue;
            }
        };
        if !matches!(prepared.kind, PreparedAssetKind::Mesh3d) {
            continue;
        }
        if let Some(source_path) = authored_mesh_source_path(&path, &prepared) {
            enrich_mesh_animation_metadata(&mut prepared.metadata, &source_path);
            source_paths.insert(source_path);
        }
        let label = prepared
            .label
            .clone()
            .or_else(|| prepared.metadata.get("label").cloned())
            .unwrap_or_else(|| discovered_label(&path));
        catalog.mark_prepared(prepared);
        output.push(DiscoveredMesh3dAsset {
            key: loaded.key,
            label,
            path,
        });
    }

    Ok((output, source_paths))
}

fn collect_glb_paths(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root)
        .map_err(|error| format!("failed to scan `{}`: {error}", root.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to scan `{}`: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_glb_paths(&path, output)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn collect_yaml_paths(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root)
        .map_err(|error| format!("failed to scan `{}`: {error}", root.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to scan `{}`: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_paths(&path, output)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
            })
        {
            output.push(path);
        }
    }
    Ok(())
}

fn authored_mesh_source_path(asset_path: &Path, prepared: &PreparedAsset) -> Option<PathBuf> {
    let source_file = prepared.metadata.get("source.file")?;
    let source_path = PathBuf::from(source_file);
    let resolved = if source_path.is_absolute() {
        source_path
    } else {
        asset_path.parent()?.join(source_path)
    };
    canonical_asset_path(&resolved)
}

fn canonical_asset_path(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}

fn enrich_mesh_animation_metadata(metadata: &mut BTreeMap<String, String>, glb_path: &Path) {
    let Some(clips) = read_glb_animation_metadata(glb_path) else {
        return;
    };
    metadata.insert("animation_clips.count".to_owned(), clips.len().to_string());
    for (index, clip) in clips.into_iter().enumerate() {
        metadata
            .entry(format!("animation_clips.{index}.label"))
            .or_insert(clip.label);
        metadata.insert(
            format!("animation_clips.{index}.duration_seconds"),
            format!("{:.4}", clip.duration_seconds.max(0.0)),
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GlbAnimationClipMetadata {
    label: String,
    duration_seconds: f32,
}

fn read_glb_animation_metadata(glb_path: &Path) -> Option<Vec<GlbAnimationClipMetadata>> {
    if !glb_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
    {
        return None;
    }
    let (document, buffers) = import_gltf_for_animation_metadata(glb_path)?;
    let clips = document
        .animations()
        .enumerate()
        .map(|(index, animation)| GlbAnimationClipMetadata {
            label: animation
                .name()
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Clip {index}")),
            duration_seconds: gltf_animation_duration_seconds(&animation, &buffers),
        })
        .collect::<Vec<_>>();
    if clips.is_empty() { None } else { Some(clips) }
}

fn import_gltf_for_animation_metadata(
    path: &Path,
) -> Option<(gltf::Document, Vec<gltf::buffer::Data>)> {
    match gltf::import(path) {
        Ok((document, buffers, _)) => Some((document, buffers)),
        Err(_) => {
            let file = fs::File::open(path).ok()?;
            let reader = std::io::BufReader::new(file);
            let gltf = gltf::Gltf::from_reader_without_validation(reader).ok()?;
            let buffers = gltf
                .blob
                .map(|blob| vec![gltf::buffer::Data(blob)])
                .unwrap_or_default();
            Some((gltf.document, buffers))
        }
    }
}

fn gltf_animation_duration_seconds(
    animation: &gltf::Animation<'_>,
    buffers: &[gltf::buffer::Data],
) -> f32 {
    animation
        .channels()
        .filter_map(|channel| {
            let reader =
                channel.reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));
            reader.read_inputs()?.last()
        })
        .fold(0.0, f32::max)
}

fn discovered_key_fragment(relative: &Path) -> String {
    relative
        .with_extension("")
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(sanitize_key_segment)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn discovered_label(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| {
            stem.replace(['_', '-'], " ")
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| "GLB Model".to_owned())
}

fn sanitize_key_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn discovers_recursive_glb_mesh_assets() {
        let root = std::env::temp_dir().join(format!("amigo-discover-glb-{}", std::process::id()));
        let mod_root = root.join("playground-npr");
        let nested = mod_root.join("source-models").join("khronos").join("male");
        fs::create_dir_all(&nested).expect("test directory should be created");
        fs::write(nested.join("hero_model.glb"), [0u8, 1, 2, 3])
            .expect("test glb should be written");

        let catalog = AssetCatalog::default();
        let discovered = discover_glb_mesh3d_assets(&catalog, &mod_root, "playground-npr")
            .expect("discovery should succeed");

        assert_eq!(discovered.len(), 1);
        assert_eq!(
            discovered[0].key.as_str(),
            "playground-npr/discovered-models/source-models/khronos/male/hero_model"
        );
        assert!(catalog.is_prepared(&discovered[0].key));
        assert_eq!(discovered_mesh3d_assets(&catalog).len(), 1);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prefers_authored_mesh_wrappers_over_raw_discovered_glb() {
        let root = std::env::temp_dir().join(format!(
            "amigo-discover-authored-mesh-{}",
            std::process::id()
        ));
        let mod_root = root.join("playground-npr");
        let mesh_root = mod_root.join("meshes");
        let source_root = mod_root.join("source-models").join("khronos");
        fs::create_dir_all(&mesh_root).expect("mesh directory should be created");
        fs::create_dir_all(&source_root).expect("source directory should be created");
        fs::write(source_root.join("Riders.glb"), [0u8, 1, 2, 3])
            .expect("test glb should be written");
        fs::write(
            mesh_root.join("riders.yml"),
            "kind: mesh-3d\nschema_version: 1\nid: riders\nlabel: Khronos Riders\nformat: glb\nsource:\n  file: ../source-models/khronos/Riders.glb\n",
        )
        .expect("test mesh wrapper should be written");

        let catalog = AssetCatalog::default();
        let discovered = discover_glb_mesh3d_assets(&catalog, &mod_root, "playground-npr")
            .expect("discovery should succeed");

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].key.as_str(), "playground-npr/meshes/riders");
        assert!(catalog.is_prepared(&discovered[0].key));
        assert_eq!(discovered_mesh3d_assets(&catalog).len(), 1);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn enriches_authored_glb_mesh_with_animation_clip_metadata() {
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();
        let mod_root = workspace_root.join("mods/playground-npr");
        let catalog = AssetCatalog::default();
        let discovered = discover_glb_mesh3d_assets(&catalog, &mod_root, "playground-npr")
            .expect("discovery should succeed");
        let soldier_key = AssetKey::new("playground-npr/meshes/soldier");

        assert!(
            discovered.iter().any(|asset| asset.key == soldier_key),
            "soldier authored mesh should be in the model library"
        );
        let prepared = catalog
            .prepared_asset(&soldier_key)
            .expect("soldier mesh should be prepared");
        assert_eq!(
            prepared
                .metadata
                .get("animation_clips.count")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            prepared
                .metadata
                .get("animation_clips.1.label")
                .map(String::as_str),
            Some("Run")
        );
        assert!(
            prepared
                .metadata
                .get("animation_clips.1.duration_seconds")
                .and_then(|value| value.parse::<f32>().ok())
                .is_some_and(|duration| duration > 0.0),
            "animation duration should be read from the GLB"
        );
    }
}
