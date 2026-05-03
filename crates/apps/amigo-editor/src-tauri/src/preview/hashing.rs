use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub fn source_hash(paths: &[&Path], width: u32, height: u32, fps: u32, frame_count: u32) -> String {
    let mut hasher = DefaultHasher::new();
    "engine-slideshow-preview-v1".hash(&mut hasher);
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    fps.hash(&mut hasher);
    frame_count.hash(&mut hasher);
    for path in paths {
        path.display().to_string().hash(&mut hasher);
        if let Ok(bytes) = fs::read(path) {
            bytes.hash(&mut hasher);
        }
    }
    format!("{:016x}", hasher.finish())
}
