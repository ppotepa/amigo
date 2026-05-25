#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleInputKind {
    Empty,
    PreferCommand,
    PreferRhai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleInputRoute {
    Empty,
    Command,
    Rhai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutedConsoleInput<'a> {
    pub route: ConsoleInputRoute,
    pub source: &'a str,
}

pub fn route_console_input(line: &str) -> RoutedConsoleInput<'_> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return RoutedConsoleInput {
            route: ConsoleInputRoute::Empty,
            source: trimmed,
        };
    }

    if let Some(source) = trimmed.strip_prefix(':') {
        return RoutedConsoleInput {
            route: ConsoleInputRoute::Rhai,
            source: source.trim_start(),
        };
    }
    if let Some(source) = trimmed.strip_prefix('=') {
        return RoutedConsoleInput {
            route: ConsoleInputRoute::Rhai,
            source: source.trim_start(),
        };
    }
    if trimmed.starts_with("inspect ") {
        return RoutedConsoleInput {
            route: ConsoleInputRoute::Command,
            source: trimmed,
        };
    }
    if looks_like_rhai(trimmed) {
        return RoutedConsoleInput {
            route: ConsoleInputRoute::Rhai,
            source: trimmed,
        };
    }

    RoutedConsoleInput {
        route: ConsoleInputRoute::Command,
        source: trimmed,
    }
}

pub fn classify_console_input(line: &str) -> ConsoleInputKind {
    match route_console_input(line).route {
        ConsoleInputRoute::Empty => ConsoleInputKind::Empty,
        ConsoleInputRoute::Command => ConsoleInputKind::PreferCommand,
        ConsoleInputRoute::Rhai => ConsoleInputKind::PreferRhai,
    }
}

pub fn should_try_rhai_route(line: &str) -> bool {
    !line.trim().is_empty()
}

pub fn looks_like_rhai(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed == "world" || trimmed.starts_with("world.") {
        return true;
    }

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
    use super::{ConsoleInputKind, ConsoleInputRoute, classify_console_input, route_console_input};

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

    #[test]
    fn routes_colon_prefix_as_rhai_without_prefix() {
        let routed = route_console_input(": let a = get_entity(\"player\")");
        assert_eq!(routed.route, ConsoleInputRoute::Rhai);
        assert_eq!(routed.source, "let a = get_entity(\"player\")");
    }

    #[test]
    fn routes_equals_prefix_as_rhai_expression_without_prefix() {
        let routed = route_console_input("= postfx.item(0).name");
        assert_eq!(routed.route, ConsoleInputRoute::Rhai);
        assert_eq!(routed.source, "postfx.item(0).name");
    }

    #[test]
    fn world_dot_expression_routes_to_rhai_eval() {
        assert_eq!(
            classify_console_input("world.camera.main.Camera2D.exposure.iso"),
            ConsoleInputKind::PreferRhai
        );
        assert_eq!(
            classify_console_input("world.camera.main.Camera2D.exposure.iso = 800"),
            ConsoleInputKind::PreferRhai
        );
        assert_eq!(
            classify_console_input("help"),
            ConsoleInputKind::PreferCommand
        );
    }

    #[test]
    fn routes_inspect_without_parentheses_as_command() {
        let routed = route_console_input("inspect entity:player");
        assert_eq!(routed.route, ConsoleInputRoute::Command);
        assert_eq!(routed.source, "inspect entity:player");
    }

    #[test]
    fn routes_inspect_expression_call_as_rhai() {
        let routed = route_console_input("inspect(entity(\"player\"))");
        assert_eq!(routed.route, ConsoleInputRoute::Rhai);
        assert_eq!(routed.source, "inspect(entity(\"player\"))");
    }

    #[test]
    fn routes_inspect_sugar_as_command() {
        let routed = route_console_input("inspect entity(\"player\")");
        assert_eq!(routed.route, ConsoleInputRoute::Command);
        assert_eq!(routed.source, "inspect entity(\"player\")");
    }
}
