use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowEventEnvelope<T>
where
    T: Serialize,
{
    pub event_id: String,
    pub event_type: String,
    pub source_window: Option<String>,
    pub session_id: Option<String>,
    pub timestamp_ms: u128,
    pub schema_version: u32,
    pub payload: T,
}
