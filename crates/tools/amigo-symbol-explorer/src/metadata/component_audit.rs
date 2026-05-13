use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct ComponentMetadataAuditReport {
    pub scene_component_variants: Vec<String>,
    pub component_kind_variants: Vec<String>,
    pub descriptor_kinds: Vec<String>,
    pub registered_descriptor_functions: Vec<String>,
    pub missing_component_kinds: Vec<String>,
    pub missing_descriptors: Vec<String>,
    pub unregistered_descriptor_kinds: Vec<String>,
}

impl ComponentMetadataAuditReport {
    pub fn print_text(&self) {
        println!("metadata-audit: components");
        println!(
            "scene_component_variants: {}",
            self.scene_component_variants.len()
        );
        println!(
            "component_kind_variants: {}",
            self.component_kind_variants.len()
        );
        println!("descriptor_kinds: {}", self.descriptor_kinds.len());
        println!(
            "registered_descriptor_functions: {}",
            self.registered_descriptor_functions.len()
        );

        print_list("missing_component_kinds", &self.missing_component_kinds);
        print_list("missing_descriptors", &self.missing_descriptors);
        print_list(
            "unregistered_descriptor_kinds",
            &self.unregistered_descriptor_kinds,
        );

        if self.missing_component_kinds.is_empty()
            && self.missing_descriptors.is_empty()
            && self.unregistered_descriptor_kinds.is_empty()
        {
            println!("status: ok");
        } else {
            println!("status: needs-work");
        }
    }
}

pub fn audit_component_metadata(root: &Path) -> Result<ComponentMetadataAuditReport> {
    let components_path = root.join("crates/engine/scene/src/document/components.rs");
    let descriptors_path = root.join("crates/engine/scene/src/component_descriptors.rs");

    let components = std::fs::read_to_string(components_path)?;
    let descriptors = std::fs::read_to_string(descriptors_path)?;

    let scene_component_variants = parse_scene_component_serde_renames(&components)?;
    let component_kind_variants = parse_component_kind_variants(&descriptors)?;
    let descriptor_kinds = parse_descriptor_kinds(&descriptors)?;
    let registered_descriptor_functions = parse_registered_descriptor_functions(&descriptors)?;

    let scene_set = scene_component_variants
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let kind_set = component_kind_variants
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let descriptor_set = descriptor_kinds.iter().cloned().collect::<BTreeSet<_>>();
    let registered_set = registered_descriptor_functions
        .iter()
        .filter_map(|function| descriptor_function_to_component_kind(function))
        .collect::<BTreeSet<_>>();

    let missing_component_kinds = scene_set.difference(&kind_set).cloned().collect::<Vec<_>>();

    let missing_descriptors = kind_set
        .difference(&descriptor_set)
        .cloned()
        .collect::<Vec<_>>();

    let unregistered_descriptor_kinds = descriptor_set
        .difference(&registered_set)
        .cloned()
        .collect::<Vec<_>>();

    Ok(ComponentMetadataAuditReport {
        scene_component_variants,
        component_kind_variants,
        descriptor_kinds,
        registered_descriptor_functions,
        missing_component_kinds,
        missing_descriptors,
        unregistered_descriptor_kinds,
    })
}

fn parse_scene_component_serde_renames(source: &str) -> Result<Vec<String>> {
    let re = regex::Regex::new(r#"#\[serde\(rename = \"([^\"]+)\"\)\]"#)?;
    let mut values = re
        .captures_iter(source)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Ok(values)
}

fn parse_component_kind_variants(source: &str) -> Result<Vec<String>> {
    let enum_body = source
        .split("pub enum ComponentKind {")
        .nth(1)
        .and_then(|tail| tail.split("}\n").next())
        .unwrap_or("");

    let re = regex::Regex::new(r#"(?m)^\s*([A-Z][A-Za-z0-9_]*),"#)?;
    let mut values = re
        .captures_iter(enum_body)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Ok(values)
}

fn parse_descriptor_kinds(source: &str) -> Result<Vec<String>> {
    let re = regex::Regex::new(r#"kind:\s*ComponentKind::([A-Za-z0-9_]+)"#)?;
    let mut values = re
        .captures_iter(source)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Ok(values)
}

fn parse_registered_descriptor_functions(source: &str) -> Result<Vec<String>> {
    let body = source
        .split("pub fn default_component_registry()")
        .nth(1)
        .unwrap_or("");
    let re = regex::Regex::new(r#"([a-z][a-z0-9_]+_descriptor)\(\)"#)?;
    let mut values = re
        .captures_iter(body)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Ok(values)
}

fn descriptor_function_to_component_kind(function_name: &str) -> Option<String> {
    let stem = function_name.strip_suffix("_descriptor")?;
    let mut output = String::new();
    for segment in stem.split('_') {
        if segment.chars().all(|ch| ch.is_ascii_digit()) {
            output.push_str(segment);
        } else if segment.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
            let digit_prefix_len = segment
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .map(char::len_utf8)
                .sum::<usize>();
            output.push_str(&segment[..digit_prefix_len]);
            output.push_str(&segment[digit_prefix_len..].to_ascii_uppercase());
        } else {
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                output.push(first.to_ascii_uppercase());
                output.push_str(chars.as_str());
            }
        }
    }
    Some(output)
}

fn print_list(label: &str, values: &[String]) {
    println!("{label}: {}", values.len());
    for value in values {
        println!("  - {value}");
    }
}

#[cfg(test)]
mod tests {
    use super::descriptor_function_to_component_kind;

    #[test]
    fn maps_descriptor_function_to_component_kind() {
        assert_eq!(
            descriptor_function_to_component_kind("sprite_2d_descriptor").as_deref(),
            Some("Sprite2D")
        );
        assert_eq!(
            descriptor_function_to_component_kind("aabb_collider_2d_descriptor").as_deref(),
            Some("AabbCollider2D")
        );
        assert_eq!(
            descriptor_function_to_component_kind("ui_document_descriptor").as_deref(),
            Some("UiDocument")
        );
    }
}

