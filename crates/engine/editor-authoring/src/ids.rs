use std::path::Path;

pub fn node_id(source_file: &Path, yaml_pointer: &str) -> String {
    let path = source_file.to_string_lossy().replace('\\', "/");
    format!("{path}#{yaml_pointer}")
}

pub fn child_pointer(parent: &str, key: &str) -> String {
    if parent.is_empty() || parent == "/" {
        format!("/{key}")
    } else {
        format!("{parent}/{key}")
    }
}
