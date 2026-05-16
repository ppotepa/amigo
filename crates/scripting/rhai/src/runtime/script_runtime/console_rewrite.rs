#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleRewriteKind {
    Unchanged,
    ControlSet,
    ControlGet,
    ControlInfo,
    ControlReset,
    ControlCommit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleRewriteResult {
    pub source: String,
    pub kind: ConsoleRewriteKind,
}

pub fn rewrite_console_source(source: &str) -> ConsoleRewriteResult {
    let trimmed = source.trim();
    if !trimmed.starts_with("world") {
        return unchanged(source);
    }

    if let Some(path) = trimmed.strip_suffix(".info()") {
        return helper("__amigo_control_info", path, None, ConsoleRewriteKind::ControlInfo);
    }
    if let Some(path) = trimmed.strip_suffix(".reset()") {
        return helper("__amigo_control_reset", path, None, ConsoleRewriteKind::ControlReset);
    }
    if let Some(path) = trimmed.strip_suffix(".commit()") {
        return helper("__amigo_control_commit", path, None, ConsoleRewriteKind::ControlCommit);
    }

    if let Some(index) = top_level_assignment_index(trimmed) {
        let path = trimmed[..index].trim();
        let rhs = trimmed[index + 1..].trim();
        if is_bare_asset_path(rhs) {
            let quoted = format!("{:?}", path);
            let rhs_quoted = format!("{:?}", rhs);
            return ConsoleRewriteResult {
                source: format!(
                    "__amigo_control_set({quoted}, __amigo_control_get({rhs_quoted}))"
                ),
                kind: ConsoleRewriteKind::ControlSet,
            };
        }
        return helper(
            "__amigo_control_set",
            path,
            Some(rhs),
            ConsoleRewriteKind::ControlSet,
        );
    }

    if is_bare_world_path(trimmed) {
        return helper("__amigo_control_get", trimmed, None, ConsoleRewriteKind::ControlGet);
    }

    unchanged(source)
}

fn helper(
    helper_name: &str,
    path: &str,
    rhs: Option<&str>,
    kind: ConsoleRewriteKind,
) -> ConsoleRewriteResult {
    let quoted = format!("{:?}", path.trim());
    let source = match rhs {
        Some(rhs) => format!("{helper_name}({quoted}, {rhs})"),
        None => format!("{helper_name}({quoted})"),
    };
    ConsoleRewriteResult { source, kind }
}

fn unchanged(source: &str) -> ConsoleRewriteResult {
    ConsoleRewriteResult {
        source: source.to_owned(),
        kind: ConsoleRewriteKind::Unchanged,
    }
}

fn is_bare_world_path(value: &str) -> bool {
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '_')
}

fn is_bare_asset_path(value: &str) -> bool {
    value.starts_with("world.assets.") && is_bare_world_path(value)
}

fn top_level_assignment_index(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut depth = 0usize;
    for (index, byte) in bytes.iter().enumerate() {
        match *byte as char {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '=' if depth == 0 => {
                let prev = index.checked_sub(1).and_then(|i| bytes.get(i)).copied();
                let next = bytes.get(index + 1).copied();
                if prev == Some(b'=') || next == Some(b'=') || prev == Some(b'!') || prev == Some(b'<') || prev == Some(b'>') {
                    continue;
                }
                return Some(index);
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_control_assignment() {
        let result = rewrite_console_source("world.weather.rain.front.ParticleEmitter2D.spawn_rate = 120");
        assert_eq!(result.kind, ConsoleRewriteKind::ControlSet);
        assert_eq!(
            result.source,
            "__amigo_control_set(\"world.weather.rain.front.ParticleEmitter2D.spawn_rate\", 120)"
        );
    }

    #[test]
    fn rewrites_control_get() {
        let result = rewrite_console_source("world.weather.rain.front");
        assert_eq!(result.kind, ConsoleRewriteKind::ControlGet);
    }

    #[test]
    fn does_not_rewrite_equality() {
        let result = rewrite_console_source("world.foo == 1");
        assert_eq!(result.kind, ConsoleRewriteKind::Unchanged);
    }

    #[test]
    fn rewrites_info_call() {
        let result = rewrite_console_source("world.foo.bar.info()");
        assert_eq!(result.kind, ConsoleRewriteKind::ControlInfo);
    }

    #[test]
    fn leaves_normal_rhai_unchanged() {
        let result = rewrite_console_source("let x = 1");
        assert_eq!(result.kind, ConsoleRewriteKind::Unchanged);
    }

    #[test]
    fn rewrites_asset_rhs_assignment() {
        let result = rewrite_console_source(
            "world.camera.main.Camera2D.film.profile = world.assets.films.neutral_digital_400",
        );
        assert_eq!(result.kind, ConsoleRewriteKind::ControlSet);
        assert_eq!(
            result.source,
            "__amigo_control_set(\"world.camera.main.Camera2D.film.profile\", __amigo_control_get(\"world.assets.films.neutral_digital_400\"))"
        );
    }
}
