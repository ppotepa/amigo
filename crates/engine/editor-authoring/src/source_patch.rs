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

/// A deliberate replacement of one complete YAML component value. The patcher
/// preserves text outside the selected value; comments inside it are rewritten.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringSourceValuePatch {
    pub source_file: PathBuf,
    /// RFC 6901-style path in the parsed source document.
    pub yaml_pointer: String,
    /// Complete value observed when editing started. Prevents stale overwrite.
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
    AmbiguousValue { matches: usize },
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
            Self::AmbiguousValue { matches } => {
                write!(f, "YAML value has {matches} matching list-item locations")
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

/// Applies an ordered scalar batch to one already-loaded source document. Each
/// replacement is validated against the original value declared by its patch;
/// callers write the returned document only after the complete batch succeeds.
pub fn patch_yaml_source_scalars(
    source: &str,
    patches: &[AuthoringSourceScalarPatch],
) -> Result<String, AuthoringSourcePatchError> {
    let mut output = source.to_owned();
    for patch in patches {
        output = patch_yaml_source_scalar(
            &output,
            &patch.yaml_pointer,
            &patch.expected,
            &patch.replacement,
        )?;
    }
    Ok(output)
}

/// Replaces one existing mapping or sequence list-item block without
/// reserializing its surrounding YAML document. This deliberately narrow
/// operation covers authored scene components; a general comment-preserving
/// YAML structural editor is not silently approximated here.
pub fn patch_yaml_source_value(
    source: &str,
    yaml_pointer: &str,
    expected: &Value,
    replacement: &Value,
) -> Result<String, AuthoringSourcePatchError> {
    if !matches!(expected, Value::Mapping(_) | Value::Sequence(_))
        || !matches!(replacement, Value::Mapping(_) | Value::Sequence(_))
    {
        return Err(AuthoringSourcePatchError::UnsupportedValue);
    }
    let parsed: Value = serde_yaml::from_str(source)
        .map_err(|error| AuthoringSourcePatchError::Parse(error.to_string()))?;
    if value_at_pointer(&parsed, yaml_pointer) != Some(expected) {
        return Err(AuthoringSourcePatchError::StaleValue);
    }
    let matches = matching_list_item_blocks(source, expected)?;
    if matches.len() != 1 {
        return Err(AuthoringSourcePatchError::AmbiguousValue {
            matches: matches.len(),
        });
    }
    let block = matches[0];
    let replacement_text = yaml_list_item_text(replacement, block.indent)?;
    let mut output = String::with_capacity(source.len() + replacement_text.len());
    output.push_str(&source[..block.start]);
    output.push_str(&replacement_text);
    output.push_str(&source[block.end..]);
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

#[derive(Debug, Clone, Copy)]
struct ListItemBlock {
    start: usize,
    end: usize,
    indent: usize,
}

fn matching_list_item_blocks(
    source: &str,
    expected: &Value,
) -> Result<Vec<ListItemBlock>, AuthoringSourcePatchError> {
    let lines = source
        .split_inclusive(['\n'])
        .scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line))
        })
        .collect::<Vec<_>>();
    let mut matches = Vec::new();
    for (index, (start, line)) in lines.iter().enumerate() {
        let body = line.trim_end_matches(['\r', '\n']);
        let indent = body.len() - body.trim_start_matches([' ', '\t']).len();
        let content = &body[indent..];
        let Some(first) = content.strip_prefix("- ") else {
            continue;
        };
        if first.trim().is_empty() || first.trim_start().starts_with('#') {
            continue;
        }
        let mut end_index = index + 1;
        while end_index < lines.len() {
            let candidate = lines[end_index].1.trim_end_matches(['\r', '\n']);
            if !candidate.trim().is_empty() && !candidate.trim_start().starts_with('#') {
                let candidate_indent = candidate.len()
                    - candidate.trim_start_matches([' ', '\t']).len();
                if candidate_indent <= indent {
                    break;
                }
            }
            end_index += 1;
        }
        let end = lines.get(end_index).map_or(source.len(), |(offset, _)| *offset);
        if list_item_value(source, *start, end, indent)? == *expected {
            matches.push(ListItemBlock {
                start: *start,
                end,
                indent,
            });
        }
    }
    Ok(matches)
}

fn list_item_value(
    source: &str,
    start: usize,
    end: usize,
    indent: usize,
) -> Result<Value, AuthoringSourcePatchError> {
    let mut output = String::new();
    for (index, line) in source[start..end].split_inclusive(['\n']).enumerate() {
        let body = line.trim_end_matches(['\r', '\n']);
        let ending = &line[body.len()..];
        if index == 0 {
            let content = body[indent..].strip_prefix("- ").ok_or_else(|| {
                AuthoringSourcePatchError::Parse("invalid YAML list item block".into())
            })?;
            output.push_str(content);
        } else {
            let child_indent = indent + 2;
            output.push_str(body.get(child_indent..).unwrap_or(body));
        }
        output.push_str(ending);
    }
    serde_yaml::from_str(&output).map_err(|error| AuthoringSourcePatchError::Parse(error.to_string()))
}

fn yaml_list_item_text(
    value: &Value,
    indent: usize,
) -> Result<String, AuthoringSourcePatchError> {
    let serialized = serde_yaml::to_string(value)
        .map_err(|error| AuthoringSourcePatchError::Parse(error.to_string()))?;
    let mut lines = serialized.lines();
    let first = lines.next().ok_or_else(|| {
        AuthoringSourcePatchError::Parse("cannot serialize empty YAML value".into())
    })?;
    let prefix = " ".repeat(indent);
    let child_prefix = " ".repeat(indent + 2);
    let mut output = format!("{prefix}- {first}\n");
    for line in lines {
        output.push_str(&child_prefix);
        output.push_str(line);
        output.push('\n');
    }
    Ok(output)
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
    fn batches_scalar_replacements_in_one_validated_document() {
        let source = "camera:\n  distance: 14.0 # framing\n  yaw: 0.0\n";
        let patch = |pointer: &str, expected: f32, replacement: f32| AuthoringSourceScalarPatch {
            source_file: "scene.yml".into(),
            yaml_pointer: pointer.to_owned(),
            expected: serde_yaml::to_value(expected).unwrap(),
            replacement: serde_yaml::to_value(replacement).unwrap(),
        };
        let output = patch_yaml_source_scalars(
            source,
            &[
                patch("/camera/distance", 14.0f32, 7.5f32),
                patch("/camera/yaw", 0.0f32, 1.25f32),
            ],
        )
        .unwrap();
        assert_eq!(output, "camera:\n  distance: 7.5 # framing\n  yaw: 1.25\n");
    }

    #[test]
    fn replaces_one_component_block_without_reformatting_the_scene() {
        let source = concat!(
            "# scene comment\n",
            "entities:\n",
            "  - id: npr\n",
            "    components:\n",
            "      # component comment is owned by the component\n",
            "      - type: amigo.gfx.npr.NprSettings\n",
            "        gallery: true\n",
            "        objects:\n",
            "          cube:\n",
            "            rotating: true\n",
            "  - id: untouched\n",
            "    name: untouched # retain\n"
        );
        let expected: Value = serde_yaml::from_str(concat!(
            "type: amigo.gfx.npr.NprSettings\n",
            "gallery: true\n",
            "objects:\n",
            "  cube:\n",
            "    rotating: true\n"
        ))
        .unwrap();
        let replacement: Value = serde_yaml::from_str(concat!(
            "type: amigo.gfx.npr.NprSettings\n",
            "gallery: true\n",
            "objects:\n",
            "  cube:\n",
            "    rotating: false\n",
            "    construction_marks:\n",
            "      - id: 7\n",
            "        anchors: []\n"
        ))
        .unwrap();
        let output = patch_yaml_source_value(
            source,
            "/entities/0/components/0",
            &expected,
            &replacement,
        )
        .unwrap();
        assert!(output.starts_with("# scene comment\nentities:\n  - id: npr\n"));
        assert!(output.contains("            rotating: false\n"));
        assert!(output.contains("  - id: untouched\n    name: untouched # retain\n"));
        let parsed: Value = serde_yaml::from_str(&output).unwrap();
        assert_eq!(value_at_pointer(&parsed, "/entities/0/components/0"), Some(&replacement));
    }

    #[test]
    fn refuses_ambiguous_component_blocks() {
        let source = concat!(
            "items:\n",
            "  - type: demo\n",
            "    enabled: true\n",
            "  - type: demo\n",
            "    enabled: true\n"
        );
        let expected: Value = serde_yaml::from_str("type: demo\nenabled: true\n").unwrap();
        let replacement: Value = serde_yaml::from_str("type: demo\nenabled: false\n").unwrap();
        assert!(matches!(
            patch_yaml_source_value(source, "/items/0", &expected, &replacement),
            Err(AuthoringSourcePatchError::AmbiguousValue { matches: 2 })
        ));
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
