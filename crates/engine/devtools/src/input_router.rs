#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleInputKind {
    Empty,
    PreferCommand,
    PreferRhai,
}

pub fn classify_console_input(line: &str) -> ConsoleInputKind {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return ConsoleInputKind::Empty;
    }

    if looks_like_rhai(trimmed) {
        return ConsoleInputKind::PreferRhai;
    }

    ConsoleInputKind::PreferCommand
}

pub fn should_try_rhai_fallback(line: &str) -> bool {
    !line.trim().is_empty()
}

pub fn looks_like_rhai(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.contains(';')
        || trimmed.contains('=')
        || trimmed.contains('{')
        || trimmed.contains('}')
        || trimmed.starts_with("let ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("if ")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("fn ")
    {
        return true;
    }

    looks_like_expression(trimmed)
}

fn looks_like_expression(line: &str) -> bool {
    if line.contains('[') || line.contains(']') {
        return true;
    }

    if line.contains('(') && line.contains(')') {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{classify_console_input, ConsoleInputKind};

    #[test]
    fn classifies_empty_input() {
        assert_eq!(classify_console_input("   "), ConsoleInputKind::Empty);
    }

    #[test]
    fn classifies_plain_command() {
        assert_eq!(
            classify_console_input("scene.entities list"),
            ConsoleInputKind::PreferCommand
        );
    }

    #[test]
    fn classifies_assignment_as_rhai() {
        assert_eq!(
            classify_console_input("get_entity(\"layer2\").opacity = 0.8"),
            ConsoleInputKind::PreferRhai
        );
    }

    #[test]
    fn classifies_index_expression_as_rhai() {
        assert_eq!(
            classify_console_input("postfx.item(0).name"),
            ConsoleInputKind::PreferRhai
        );
    }
}
