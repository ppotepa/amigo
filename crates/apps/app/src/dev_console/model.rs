#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedConsoleCommand {
    pub(crate) raw: String,
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConsoleCommandDescriptor {
    pub(crate) name: &'static str,
    pub(crate) aliases: &'static [&'static str],
    pub(crate) category: &'static str,
    pub(crate) help: &'static str,
    pub(crate) usage: &'static str,
    pub(crate) examples: &'static [&'static str],
    pub(crate) dev_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConsoleCommandResult {
    Ok(String),
    Error(String),
    Silent,
    Unknown(String),
}

impl ConsoleCommandResult {
    pub(crate) fn ok(message: impl Into<String>) -> Self {
        Self::Ok(message.into())
    }

    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    pub(crate) fn unknown(raw: impl Into<String>) -> Self {
        Self::Unknown(raw.into())
    }
}
