//! Validated local imports and trusted, hash-verified remote artifacts.
use crate::documents::{authored_path, validate_document_id};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
const MAX_ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprCatalogEndpoint {
    pub id: String,
    pub url: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprCatalogsDocument {
    #[serde(default)]
    pub catalogs: Vec<NprCatalogEndpoint>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprRemoteCatalog {
    pub models: Vec<NprRemoteModel>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprRemoteModel {
    pub id: String,
    pub version: String,
    pub entry: String,
    pub files: Vec<NprRemoteFile>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprRemoteFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}
#[derive(Debug, Clone)]
pub struct NprImportedModel {
    pub id: String,
    pub path: PathBuf,
    pub byte_len: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelManifest {
    id: String,
    entry: PathBuf,
    byte_len: u64,
}

pub fn imported_models(root: &Path) -> Result<Vec<NprImportedModel>, String> {
    let parent = authored_path(root, Path::new("assets/models"))?;
    if !parent.exists() {
        return Ok(vec![]);
    }
    let mut models = Vec::new();
    for entry in fs::read_dir(&parent).map_err(|e| e.to_string())? {
        let folder = entry.map_err(|e| e.to_string())?.path();
        let manifest = folder.join("npr-model.json");
        if !manifest.exists() {
            continue;
        }
        let folder = folder.canonicalize().map_err(|e| e.to_string())?;
        if !folder.starts_with(parent.canonicalize().map_err(|e| e.to_string())?) {
            return Err("model folder escapes active mod".into());
        }
        let manifest: ModelManifest = serde_json::from_slice(&read_bounded(&manifest, 16 * 1024)?)
            .map_err(|e| e.to_string())?;
        validate_document_id(&manifest.id)?;
        let folder_id = folder
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("model folder has no valid identifier")?;
        if manifest.id != folder_id {
            return Err("model manifest id does not match its folder".into());
        }
        if crate::state::MODELS.contains(&manifest.id.as_str()) {
            return Err("imported model cannot override a built-in model".into());
        }
        let path = authored_path(&folder, &manifest.entry)?;
        models.push(NprImportedModel {
            id: manifest.id,
            path,
            byte_len: manifest.byte_len,
        });
    }
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("model artifact exceeds size limit".into());
    }
    Ok(bytes)
}

fn json_document(path: &Path, bytes: &[u8]) -> Result<serde_json::Value, String> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("gltf") => serde_json::from_slice(bytes).map_err(|e| e.to_string()),
        Some("glb") => {
            if bytes.len() < 20
                || &bytes[0..4] != b"glTF"
                || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
                || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
                || &bytes[16..20] != b"JSON"
            {
                return Err("invalid GLB header".into());
            }
            let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
            let json = bytes
                .get(20..20usize.checked_add(count).ok_or("invalid GLB chunk")?)
                .ok_or("invalid GLB JSON chunk")?;
            serde_json::from_slice(json).map_err(|e| e.to_string())
        }
        _ => Err("only GLB and glTF models are supported".into()),
    }
}

fn relative_uri(uri: &str) -> Result<PathBuf, String> {
    let decoded = percent_encoding::percent_decode_str(uri)
        .decode_utf8()
        .map_err(|e| e.to_string())?;
    if decoded.contains([':', '\\', '?', '#', '\0']) {
        return Err("invalid external model URI".into());
    }
    let path = Path::new(decoded.as_ref());
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("model reference escapes import folder".into());
    }
    Ok(path.to_owned())
}

fn artifact_files(entry: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let entry = entry.canonicalize().map_err(|e| e.to_string())?;
    let root = entry.parent().ok_or("model has no import folder")?;
    let bytes = read_bounded(&entry, MAX_ARTIFACT_BYTES)?;
    let document = json_document(&entry, &bytes)?;
    let mut paths = BTreeSet::new();
    for collection in ["buffers", "images"] {
        for item in document
            .get(collection)
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(uri) = item.get("uri").and_then(|v| v.as_str()) {
                if uri.starts_with("data:") {
                    continue;
                }
                paths.insert(relative_uri(uri)?);
            }
        }
    }
    let mut total = bytes.len() as u64;
    let mut files = vec![(
        PathBuf::from(entry.file_name().ok_or("missing model filename")?),
        bytes,
    )];
    for path in paths {
        let full = root.join(&path).canonicalize().map_err(|e| e.to_string())?;
        if !full.starts_with(root) {
            return Err("model reference escapes import folder".into());
        }
        let bytes = read_bounded(&full, MAX_ARTIFACT_BYTES.saturating_sub(total))?;
        total += bytes.len() as u64;
        files.push((path, bytes));
    }
    // Decode geometry only after every external URI has been checked.
    amigo_3d_mesh::load_gltf_geometry(&entry)?;
    Ok(files)
}

pub fn import_local(root: &Path, entry: &Path) -> Result<NprImportedModel, String> {
    let files = artifact_files(entry)?;
    let mut hash = Sha256::new();
    for (path, bytes) in &files {
        hash.update(path.to_string_lossy().as_bytes());
        hash.update([0]);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    let digest = hash.finalize();
    let id = format!(
        "import-{}",
        digest[..8]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    import_files(root, &id, files)
}

fn import_files(
    root: &Path,
    id: &str,
    files: Vec<(PathBuf, Vec<u8>)>,
) -> Result<NprImportedModel, String> {
    validate_document_id(id)?;
    let parent = authored_path(root, Path::new("assets/models"))?;
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let target = parent.join(id);
    if target.exists() {
        return Err("model already imported; choose a distinct artifact version".into());
    }
    let staging = tempfile::tempdir_in(&parent).map_err(|e| e.to_string())?;
    let mut total = 0;
    for (path, bytes) in &files {
        let destination = authored_path(staging.path(), path)?;
        fs::create_dir_all(destination.parent().ok_or("invalid artifact path")?)
            .map_err(|e| e.to_string())?;
        fs::write(destination, bytes).map_err(|e| e.to_string())?;
        total += bytes.len() as u64;
    }
    let manifest = ModelManifest {
        id: id.into(),
        entry: files[0].0.clone(),
        byte_len: total,
    };
    fs::write(
        staging.path().join("npr-model.json"),
        serde_json::to_vec(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    // All-or-nothing publication. A partial artifact is never an authored model.
    fs::rename(staging.path(), &target).map_err(|e| e.to_string())?;
    Ok(NprImportedModel {
        id: id.into(),
        path: target.join(&files[0].0),
        byte_len: total,
    })
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())
}

fn trusted_url(endpoint: &NprCatalogEndpoint) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(&endpoint.url).map_err(|e| e.to_string())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("catalog endpoint must use HTTP(S) without credentials".into());
    }
    Ok(url)
}

pub fn fetch_catalog(endpoint: &NprCatalogEndpoint) -> Result<NprRemoteCatalog, String> {
    let mut bytes = Vec::new();
    client()?
        .get(trusted_url(endpoint)?)
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("catalog exceeds size limit".into());
    }
    let catalog: NprRemoteCatalog = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    for model in &catalog.models {
        validate_remote_model(model)?;
    }
    Ok(catalog)
}

fn validate_remote_model(model: &NprRemoteModel) -> Result<(), String> {
    validate_document_id(&model.id)?;
    validate_document_id(&model.version)?;
    let entry = relative_uri(&model.entry)?;
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for file in &model.files {
        let path = relative_uri(&file.path)?;
        if !seen.insert(path) {
            return Err("duplicate artifact file".into());
        }
        total = total
            .checked_add(file.size)
            .ok_or("artifact size overflow")?;
        if total > MAX_ARTIFACT_BYTES
            || file.sha256.len() != 64
            || !file.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("invalid artifact size or SHA-256".into());
        }
    }
    if !seen.contains(&entry) {
        return Err("artifact entry missing from manifest".into());
    }
    Ok(())
}

pub fn import_remote(
    root: &Path,
    cache: &Path,
    endpoint: &NprCatalogEndpoint,
    model: &NprRemoteModel,
) -> Result<NprImportedModel, String> {
    validate_remote_model(model)?;
    let base = trusted_url(endpoint)?;
    let client = client()?;
    fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let staging = tempfile::tempdir_in(cache).map_err(|e| e.to_string())?;
    for file in &model.files {
        let path = relative_uri(&file.path)?;
        let url = base.join(&file.path).map_err(|e| e.to_string())?;
        if url.origin() != base.origin() {
            return Err("artifact URL leaves trusted endpoint".into());
        }
        let cache_file = cache.join(file.sha256.to_ascii_lowercase());
        let cached = read_bounded(&cache_file, file.size).ok().filter(|bytes| {
            bytes.len() as u64 == file.size
                && format!("{:x}", Sha256::digest(bytes)).eq_ignore_ascii_case(&file.sha256)
        });
        let cache_valid = cached.is_some();
        let mut bytes = cached.unwrap_or_default();
        if bytes.is_empty() && file.size != 0 {
            client
                .get(url)
                .send()
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .take(file.size + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
        }
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if bytes.len() as u64 != file.size || !hash.eq_ignore_ascii_case(&file.sha256) {
            return Err(format!("artifact size/hash mismatch: {}", file.path));
        }
        if !cache_valid {
            use std::io::Write;
            let mut cached = tempfile::NamedTempFile::new_in(cache).map_err(|e| e.to_string())?;
            cached.write_all(&bytes).map_err(|e| e.to_string())?;
            cached.persist(&cache_file).map_err(|e| e.to_string())?;
        }
        let target = authored_path(staging.path(), &path)?;
        fs::create_dir_all(target.parent().ok_or("invalid artifact directory")?)
            .map_err(|e| e.to_string())?;
        fs::write(target, bytes).map_err(|e| e.to_string())?;
    }
    let entry = staging.path().join(relative_uri(&model.entry)?);
    let files = artifact_files(&entry)?;
    // artifact_files requires every referenced dependency to exist in the verified staging directory.
    let id = format!("{}-{}", model.id, model.version);
    import_files(root, &id, files)
}
