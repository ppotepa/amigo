use super::model::ParsedConsoleCommand;

pub(crate) fn parse_console_command(line: &str) -> Option<ParsedConsoleCommand> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts = trimmed
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let name = parts.first()?.clone();
    let args = parts.into_iter().skip(1).collect();

    Some(ParsedConsoleCommand {
        raw: trimmed.to_owned(),
        name,
        args,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_console_command;

    #[test]
    fn parses_simple_console_command() {
        let parsed = parse_console_command("particles.emitter rain max 40").unwrap();
        assert_eq!(parsed.name, "particles.emitter");
        assert_eq!(parsed.args, vec!["rain", "max", "40"]);
    }
}
