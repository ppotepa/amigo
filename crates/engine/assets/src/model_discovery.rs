use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    AssetCatalog, AssetKey, AssetManifest, AssetSourceKind, LoadedAsset, PreparedAsset,
    PreparedAssetKind,
};

pub const DISCOVERED_MESH_3D_TAG: &str = "discovered-mesh-3d";

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
    let source_root = mod_root.join("source-models");
    if !source_root.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    collect_glb_paths(&source_root, &mut paths)?;
    paths.sort();

    let mut output = Vec::new();
    for path in paths {
        let Some(relative) = path.strip_prefix(mod_root).ok() else {
            continue;
        };
        let asset_key = AssetKey::new(format!(
            "{mod_id}/discovered-models/{}",
            discovered_key_fragment(relative)
        ));
        let label = discovered_label(&path);
        let byte_len = fs::metadata(&path).map(|metadata| metadata.len()).unwrap_or(0);
        let mut metadata = BTreeMap::new();
        metadata.insert("kind".to_owned(), "mesh-3d".to_owned());
        metadata.insert("format".to_owned(), "glb".to_owned());
        metadata.insert("label".to_owned(), label.clone());
        metadata.insert("source.file".to_owned(), path.display().to_string());
        metadata.insert(
            "source.discovered_from".to_owned(),
            source_root.display().to_string(),
        );

        catalog.register_manifest(AssetManifest {
            key: asset_key.clone(),
            source: AssetSourceKind::Mod(mod_id.to_owned()),
            tags: vec![
                "mesh-3d".to_owned(),
                "model-3d".to_owned(),
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
                    .any(|tag| tag == DISCOVERED_MESH_3D_TAG)
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
        let root = std::env::temp_dir().join(format!(
            "amigo-discover-glb-{}",
            std::process::id()
        ));
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
}
