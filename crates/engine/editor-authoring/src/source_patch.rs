//! Conservative, format-preserving scalar edits for authored YAML documents.
//!
//! The editor deliberately refuses mappings, sequences, aliases and ambiguous
//! keys.  Structural editing needs a CST-based writer; silently reserializing a
//! whole scene would discard comments and author formatting.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringSourceScalarPatch {
    pub source_file: PathBuf,
    /// RFC 6901-style path in the parsed source document.
    pub yaml_pointer: String,
    /// Value observed when the edit was started. Prevents stale overwrites.
    pub expected: Value,
    pub replacement: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoringSourcePatchError {
    InvalidPointer(String),
    UnsupportedValue,
    Parse(String),
    StaleValue,
    AmbiguousScalar { key: String, matches: usize },
    Io(String),
}

impl std::fmt::Display for AuthoringSourcePatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPointer(pointer) => write!(f, "invalid YAML pointer `{pointer}`"),
            Self::UnsupportedValue => f.write_str("source scalar patch requires scalar values"),
            Self::Parse(error) => write!(f, "cannot parse authored YAML: {error}"),
            Self::StaleValue => f.write_str("authored YAML changed since this edit was started"),
            Self::AmbiguousScalar { key, matches } => {
                write!(f, "YAML key `{key}` has {matches} matching scalar locations")
            }
            Self::Io(error) => write!(f, "source patch I/O error: {error}"),
        }
    }
}

impl std::error::Error for AuthoringSourcePatchError {}

/// Applies a single existing scalar replacement without reserializing the
/// surrounding YAML. The returned text is validated against the pointer before
/// it is eligible for disk persistence.
pub fn patch_yaml_source_scalar(
    source: &str,
    yaml_pointer: &str,
    expected: &Value,
    replacement: &Value,
) -> Result<String, AuthoringSourcePatchError> {
    if !is_scalar(replacement) {
        return Err(AuthoringSourcePatchError::UnsupportedValue);
    }
    let parsed: Value = serde_yaml::from_str(source)
        .map_err(|error| AuthoringSourcePatchError::Parse(error.to_string()))?;
    if value_at_pointer(&parsed, yaml_pointer) != Some(expected) {
        return Err(AuthoringSourcePatchError::StaleValue);
    }
    let key = pointer_key(yaml_pointer)?;
    let replacement_text = scalar_yaml_text(replacement)?;
    let mut matches = Vec::new();
    let mut offset = 0usize;
    for line in source.split_inclusive(['\n']) {
        if scalar_line_value(line, &key).is_some_and(|value| value == *expected) {
            matches.push((offset, line.len()));
        }
        offset += line.len();
    }
    if matches.len() != 1 {
        return Err(AuthoringSourcePatchError::AmbiguousScalar {
            key,
            matches: matches.len(),
        });
    }
    let (start, len) = matches[0];
    let line = &source[start..start + len];
    let replacement_line = replace_scalar_line(line, &replacement_text)?;
    let mut output = String::with_capacity(source.len() + replacement_line.len());
    output.push_str(&source[..start]);
    output.push_str(&replacement_line);
    output.push_str(&source[start + len..]);
    let reparsed: Value = serde_yaml::from_str(&output)
        .map_err(|error| AuthoringSourcePatchError::Parse(error.to_string()))?;
    if value_at_pointer(&reparsed, yaml_pointer) != Some(replacement) {
        return Err(AuthoringSourcePatchError::StaleValue);
    }
    Ok(output)
}

/// Atomically replaces an existing authored file with validated source text.
/// If the final rename cannot be performed, the old document remains intact.
pub fn write_yaml_source_atomically(
    path: &Path,
    source: &str,
) -> Result<(), AuthoringSourcePatchError> {
    let parent = path.parent().ok_or_else(|| {
        AuthoringSourcePatchError::Io("source document has no parent directory".into())
    })?;
    let stem = path.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        AuthoringSourcePatchError::Io("source document has no UTF-8 filename".into())
    })?;
    let mut temporary = None;
    for attempt in 0..64u32 {
        let candidate = parent.join(format!(".{stem}.amigo-edit-{}-{attempt}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(mut file) => {
                file.write_all(source.as_bytes())
                    .and_then(|_| file.sync_all())
                    .map_err(|error| AuthoringSourcePatchError::Io(error.to_string()))?;
                temporary = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(AuthoringSourcePatchError::Io(error.to_string())),
        }
    }
    let temporary = temporary.ok_or_else(|| {
        AuthoringSourcePatchError::Io("cannot allocate a source-edit temporary file".into())
    })?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(AuthoringSourcePatchError::Io(error.to_string()));
    }
    Ok(())
}

fn is_scalar(value: &Value) -> bool {
    matches!(value, Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_))
}

fn pointer_key(pointer: &str) -> Result<String, AuthoringSourcePatchError> {
    let key = pointer
        .strip_prefix('/')
        .and_then(|pointer| pointer.rsplit('/').next())
        .filter(|key| !key.is_empty())
        .ok_or_else(|| AuthoringSourcePatchError::InvalidPointer(pointer.to_owned()))?;
    let key = decode_pointer_segment(key)?;
    if key.parse::<usize>().is_ok() || key.contains(':') || key.contains('\n') {
        return Err(AuthoringSourcePatchError::InvalidPointer(pointer.to_owned()));
    }
    Ok(key)
}

fn value_at_pointer<'a>(value: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        return Some(value);
    }
    pointer.strip_prefix('/')?.split('/').try_fold(value, |value, segment| {
        let segment = decode_pointer_segment(segment).ok()?;
        match value {
            Value::Mapping(map) => map.get(Value::String(segment)),
            Value::Sequence(values) => values.get(segment.parse::<usize>().ok()?),
            _ => None,
        }
    })
}

fn decode_pointer_segment(segment: &str) -> Result<String, AuthoringSourcePatchError> {
    let mut output = String::new();
    let mut chars = segment.chars();
    while let Some(character) = chars.next() {
        if character != '~' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('0') => output.push('~'),
            Some('1') => output.push('/'),
            _ => return Err(AuthoringSourcePatchError::InvalidPointer(segment.to_owned())),
        }
    }
    Ok(output)
}

fn scalar_yaml_text(value: &Value) -> Result<String, AuthoringSourcePatchError> {
    let text = serde_yaml::to_string(value)
        .map_err(|error| AuthoringSourcePatchError::Parse(error.to_string()))?;
    let text = text.trim();
    if text.contains('\n') || text.starts_with('-') || text.starts_with('{') || text.starts_with('[') {
        return Err(AuthoringSourcePatchError::UnsupportedValue);
    }
    Ok(text.to_owned())
}

fn scalar_line_value(line: &str, key: &str) -> Option<Value> {
    let content = line.trim_start();
    if content.starts_with('-') || content.starts_with('#') {
        return None;
    }
    let remainder = content.strip_prefix(key)?.strip_prefix(':')?.trim_start();
    if remainder.is_empty() || remainder.starts_with('|') || remainder.starts_with('>') {
        return None;
    }
    let value = remainder[..comment_start(remainder).unwrap_or(remainder.len())].trim_end();
    let parsed: Value = serde_yaml::from_str::<Value>(&format!("value: {value}"))
        .ok()?
        .as_mapping()?
        .get(Value::String("value".into()))?
        .clone();
    is_scalar(&parsed).then_some(parsed)
}

fn replace_scalar_line(line: &str, replacement: &str) -> Result<String, AuthoringSourcePatchError> {
    let ending = if line.ends_with("\r\n") { "\r\n" } else if line.ends_with('\n') { "\n" } else { "" };
    let body = line.strip_suffix(ending).unwrap_or(line);
    let colon = body.find(':').ok_or_else(|| AuthoringSourcePatchError::Parse("scalar line has no colon".into()))?;
    let after_colon = &body[colon + 1..];
    let leading = after_colon.len() - after_colon.trim_start().len();
    let value_part = &after_colon[leading..];
    let comment = comment_start(value_part)
        .map(|index| {
            let scalar_end = value_part[..index].trim_end().len();
            &value_part[scalar_end..]
        })
        .unwrap_or("");
    Ok(format!("{}:{}{}{}{}", &body[..colon], &after_colon[..leading], replacement, comment, ending))
}

fn comment_start(value: &str) -> Option<usize> {
    let mut quoted = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(quote) = quoted {
            if quote == '"' && character == '\\' && !escaped {
                escaped = true;
                continue;
            }
            if character == quote && !escaped {
                quoted = None;
            }
            escaped = false;
        } else if matches!(character, '\'' | '"') {
            quoted = Some(character);
        } else if character == '#'
            && (index == 0 || value[..index].chars().last().is_some_and(char::is_whitespace))
        {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_one_scalar_without_reformatting_its_comment_or_neighbours() {
        let source = "# authored heading\ncamera:\n  distance: 14.0 # framing\n  yaw: 0\n";
        let output = patch_yaml_source_scalar(
            source,
            "/camera/distance",
            &serde_yaml::to_value(14.0).unwrap(),
            &serde_yaml::to_value(7.5).unwrap(),
        )
        .unwrap();
        assert_eq!(output, "# authored heading\ncamera:\n  distance: 7.5 # framing\n  yaw: 0\n");
    }

    #[test]
    fn refuses_ambiguous_scalar_locations() {
        let source = "left:\n  enabled: true\nright:\n  enabled: true\n";
        assert!(matches!(
            patch_yaml_source_scalar(
                source,
                "/left/enabled",
                &Value::Bool(true),
                &Value::Bool(false),
            ),
            Err(AuthoringSourcePatchError::AmbiguousScalar { matches: 2, .. })
        ));
    }

    #[test]
    fn refuses_a_stale_document_value() {
        let source = "gallery: true\n";
        assert!(matches!(
            patch_yaml_source_scalar(source, "/gallery", &Value::Bool(false), &Value::Bool(true)),
            Err(AuthoringSourcePatchError::StaleValue)
        ));
    }

    #[test]
    fn atomically_replaces_a_validated_temporary_document() {
        let root = std::env::temp_dir().join(format!(
            "amigo-source-patch-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let file = root.join("scene.yml");
        fs::write(&file, "gallery: false # before\n").unwrap();
        let output = patch_yaml_source_scalar(
            &fs::read_to_string(&file).unwrap(),
            "/gallery",
            &Value::Bool(false),
            &Value::Bool(true),
        )
        .unwrap();
        write_yaml_source_atomically(&file, &output).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "gallery: true # before\n");
        fs::remove_dir_all(root).unwrap();
    }
}
