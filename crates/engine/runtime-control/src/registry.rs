use std::collections::BTreeMap;

use crate::{
    ControlRange, ControlValueType, RuntimeControlCompletionEntry, RuntimeControlCompletionKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlScope {
    RuntimeLive,
    SourceDocument,
    RuntimeAndSource,
}

#[derive(Debug, Clone)]
pub struct RuntimeControlProperty {
    pub console_path: String,
    pub target_path: String,
    pub component: Option<String>,
    pub property_path: String,
    pub value_type: ControlValueType,
    pub range: Option<ControlRange>,
    pub writable: bool,
    pub readable: bool,
    pub animatable: bool,
    pub source_file: Option<String>,
    pub source_pointer: Option<String>,
    pub provider_id: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeControlTarget {
    pub console_path: String,
    pub source_id: Option<String>,
    pub label: String,
    pub components: Vec<String>,
    pub aliases: Vec<String>,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeControlRegistry {
    pub(crate) targets_by_path: BTreeMap<String, RuntimeControlTarget>,
    pub(crate) properties_by_path: BTreeMap<String, RuntimeControlProperty>,
    pub(crate) aliases: BTreeMap<String, String>,
}

impl RuntimeControlRegistry {
    pub fn register_target(&mut self, target: RuntimeControlTarget) {
        self.targets_by_path
            .insert(target.console_path.clone(), target.clone());
        for alias in &target.aliases {
            self.register_alias(alias.clone(), target.console_path.clone());
        }
    }

    pub fn register_alias(&mut self, alias: impl Into<String>, canonical: impl Into<String>) {
        self.aliases.insert(alias.into(), canonical.into());
    }

    pub fn register_property(&mut self, property: RuntimeControlProperty) {
        self.properties_by_path
            .insert(property.console_path.clone(), property);
    }

    pub fn target(&self, path: &str) -> Option<&RuntimeControlTarget> {
        self.resolve_target(path)
            .and_then(|resolved| self.targets_by_path.get(resolved.as_str()))
    }

    pub fn property(&self, path: &str) -> Option<&RuntimeControlProperty> {
        self.resolve_property(path)
            .and_then(|resolved| self.properties_by_path.get(resolved.as_str()))
    }

    pub fn resolve_alias(&self, path: &str) -> String {
        self.aliases
            .get(path)
            .cloned()
            .unwrap_or_else(|| path.to_owned())
    }

    pub fn children_of(&self, prefix: &str) -> Vec<RuntimeControlCompletionEntry> {
        let normalized = prefix.trim_end_matches('.');
        let path_prefix = if normalized.is_empty() {
            "world".to_owned()
        } else {
            normalized.to_owned()
        };
        let mut children = BTreeMap::<String, RuntimeControlCompletionEntry>::new();

        for path in self
            .targets_by_path
            .keys()
            .chain(self.properties_by_path.keys())
        {
            if path == &path_prefix || !path.starts_with(path_prefix.as_str()) {
                continue;
            }
            let suffix = path[path_prefix.len()..].trim_start_matches('.');
            if suffix.is_empty() {
                continue;
            }
            let first = suffix.split('.').next().unwrap();
            let child_path = format!("{path_prefix}.{first}");
            let kind = if self.targets_by_path.contains_key(child_path.as_str()) {
                RuntimeControlCompletionKind::Target
            } else if self.properties_by_path.contains_key(child_path.as_str()) {
                RuntimeControlCompletionKind::Property
            } else {
                RuntimeControlCompletionKind::Namespace
            };
            children
                .entry(first.to_owned())
                .or_insert_with(|| RuntimeControlCompletionEntry {
                    label: first.to_owned(),
                    insert_text: match kind {
                        RuntimeControlCompletionKind::Property => first.to_owned(),
                        _ => format!("{first}."),
                    },
                    path: child_path,
                    kind,
                    detail: None,
                });
        }

        children.into_values().collect()
    }

    pub fn properties_for_target_component(
        &self,
        target: &str,
        component: &str,
    ) -> Vec<RuntimeControlProperty> {
        self.properties_by_path
            .values()
            .filter(|property| {
                property.target_path == target && property.component.as_deref() == Some(component)
            })
            .cloned()
            .collect()
    }

    fn resolve_target(&self, path: &str) -> Option<String> {
        let canonical = self.resolve_alias(path);
        self.targets_by_path
            .contains_key(canonical.as_str())
            .then_some(canonical)
    }

    fn resolve_property(&self, path: &str) -> Option<String> {
        let canonical = self.resolve_alias(path);
        self.properties_by_path
            .contains_key(canonical.as_str())
            .then_some(canonical)
    }
}
