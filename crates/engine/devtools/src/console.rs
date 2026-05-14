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

    let parts = tokenize_console_line(trimmed);
    let name = parts.first()?.clone();
    let args = parts.into_iter().skip(1).collect();

    Some(ParsedConsoleCommand {
        raw: trimmed.to_owned(),
        name,
        args,
    })
}

pub fn tokenize_console_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            token.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                token.push(ch);
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            ch if ch.is_whitespace() => {
                if !token.is_empty() {
                    tokens.push(std::mem::take(&mut token));
                }
            }
            _ => token.push(ch),
        }
    }

    if escaped {
        token.push('\\');
    }

    if !token.is_empty() {
        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::{parse_console_command, tokenize_console_line};

    #[test]
    fn parses_simple_console_command() {
        let parsed = parse_console_command("particles.emitter rain max 40").unwrap();
        assert_eq!(parsed.name, "particles.emitter");
        assert_eq!(parsed.args, vec!["rain", "max", "40"]);
    }

    #[test]
    fn tokenizes_quoted_arguments() {
        let tokens = tokenize_console_line("scene.entities add \"enemy boss\"");
        assert_eq!(tokens, vec!["scene.entities", "add", "enemy boss"]);
    }

    #[test]
    fn tokenizes_escaped_quotes() {
        let tokens = tokenize_console_line("echo \"hello \\\"world\\\"\"");
        assert_eq!(tokens, vec!["echo", "hello \"world\""]);
    }

    #[test]
    fn parses_canonical_console_command() {
        let parsed = parse_console_command("scene.entities add \"enemy boss\"").unwrap();
        assert_eq!(parsed.name, "scene.entities");
        assert_eq!(parsed.args, vec!["add", "enemy boss"]);
    }
}
