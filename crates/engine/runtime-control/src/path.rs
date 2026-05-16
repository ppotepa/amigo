use crate::{RuntimeControlError, RuntimeControlSceneMetadata};

pub const WORLD_PREFIX: &str = "world";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsoleControlPath {
    pub raw: String,
    pub segments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedControlPath {
    pub target_path: String,
    pub component: Option<String>,
    pub property_path: Option<String>,
}

impl ConsoleControlPath {
    pub fn parse(input: &str) -> Result<Self, RuntimeControlError> {
        let trimmed = input.trim().trim_end_matches('.');
        let segments = trimmed
            .split('.')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if segments.first().map(String::as_str) != Some(WORLD_PREFIX) {
            return Err(RuntimeControlError::InvalidPath(input.to_owned()));
        }
        Ok(Self {
            raw: trimmed.to_owned(),
            segments,
        })
    }

    pub fn after_world(&self) -> &[String] {
        self.segments.get(1..).unwrap_or(&[])
    }

    pub fn last_segment(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }

    pub fn resolve(
        &self,
        metadata: &RuntimeControlSceneMetadata,
    ) -> Result<ResolvedControlPath, RuntimeControlError> {
        let after_world = self.after_world();
        if after_world.is_empty() {
            return Ok(ResolvedControlPath {
                target_path: WORLD_PREFIX.to_owned(),
                component: None,
                property_path: None,
            });
        }

        let joined = after_world.join(".");
        if metadata.target_lookup.contains_key(&joined) {
            return Ok(ResolvedControlPath {
                target_path: joined,
                component: None,
                property_path: None,
            });
        }

        for split in (1..after_world.len()).rev() {
            let target = after_world[..split].join(".");
            let component = &after_world[split];
            if metadata.target_lookup.contains_key(&target)
                && metadata.is_known_component(component)
            {
                let property_path = if split + 1 < after_world.len() {
                    Some(after_world[split + 1..].join("."))
                } else {
                    None
                };
                return Ok(ResolvedControlPath {
                    target_path: target,
                    component: Some(component.clone()),
                    property_path,
                });
            }
        }

        Err(RuntimeControlError::InvalidPath(self.raw.clone()))
    }
}

pub fn sanitize_console_segment(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

pub fn split_console_prefix(input: &str) -> Option<(String, String)> {
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return None;
    }
    let dot = trimmed.rfind('.')?;
    Some((trimmed[..dot].to_owned(), trimmed[dot + 1..].to_owned()))
}
