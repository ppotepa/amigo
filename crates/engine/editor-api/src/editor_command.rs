#[derive(Debug, Clone)]
pub struct EditorCommand {
    pub id: String,
    pub payload: EditorCommandPayload,
}

#[derive(Debug, Clone)]
pub enum EditorCommandPayload {
    Empty,
    Text(String),
}
