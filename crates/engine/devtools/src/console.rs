#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConsoleCommand {
    pub raw: String,
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConsoleCommandDescriptor {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub category: &'static str,
    pub help: &'static str,
    pub usage: &'static str,
    pub examples: &'static [&'static str],
    pub dev_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleCommandResult {
    Ok(String),
    Error(String),
    Silent,
    Unknown(String),
}

impl ConsoleCommandResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self::Ok(message.into())
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    pub fn unknown(raw: impl Into<String>) -> Self {
        Self::Unknown(raw.into())
    }
}

pub fn parse_console_command(line: &str) -> Option<ParsedConsoleCommand> {
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

