//! Backend-neutral contracts for opt-in scene playgrounds.
mod host;
mod viewport;
pub use host::*;
pub use viewport::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const PLAYGROUND_PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlaygroundId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundDescriptor {
    pub id: PlaygroundId,
    pub label: String,
    /// Client asset registration key, interpreted only by the companion host.
    pub client: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundSnapshot {
    pub revision: u64,
    pub values: BTreeMap<String, Value>,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundActionEnvelope {
    pub request_id: u64,
    pub base_revision: u64,
    pub control: String,
    pub intent: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundActionError {
    pub control: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlaygroundEvent {
    Snapshot {
        snapshot: PlaygroundSnapshot,
    },
    Delta {
        base_revision: u64,
        revision: u64,
        changed: BTreeMap<String, Value>,
        removed: Vec<String>,
        metadata: Option<Value>,
    },
    Accepted {
        request_id: u64,
        revision: u64,
    },
    Rejected {
        request_id: u64,
        error: PlaygroundActionError,
    },
    Domain {
        name: String,
        payload: Value,
    },
    Closed,
}

pub trait PlaygroundProvider: Send + Sync {
    fn descriptor(&self) -> PlaygroundDescriptor;
    fn snapshot(&self) -> PlaygroundSnapshot;
    fn revision(&self) -> u64 { self.snapshot().revision }
    fn cancel_interaction(&self) {}
    fn open_scene(
        &self,
        _mod_root: &std::path::Path,
        _scene_path: &std::path::Path,
    ) -> Result<(), String> {
        Ok(())
    }
    /// Must validate completely before mutating. Called serially by the host.
    fn dispatch(&self, action: &PlaygroundActionEnvelope) -> Result<(), PlaygroundActionError>;
    fn drain_events(&self) -> Vec<PlaygroundEvent> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundHandshake {
    pub version: u32,
    pub playground: PlaygroundId,
    pub token: String,
}

/// A failed attempt never consumes the token; a successful one cannot be replayed.
pub struct PlaygroundHandshakeGuard {
    playground: PlaygroundId,
    token: Option<String>,
    origin: String,
}

impl PlaygroundHandshakeGuard {
    pub fn new(playground: PlaygroundId, token: String, origin: String) -> Self {
        Self {
            playground,
            token: Some(token),
            origin,
        }
    }

    pub fn accept(
        &mut self,
        hello: &PlaygroundHandshake,
        origin: &str,
    ) -> Result<(), &'static str> {
        if hello.version != PLAYGROUND_PROTOCOL_VERSION {
            return Err("protocol_version");
        }
        if hello.playground != self.playground {
            return Err("playground_id");
        }
        if origin != self.origin {
            return Err("origin");
        }
        let Some(token) = &self.token else {
            return Err("token_consumed");
        };
        let difference = token
            .bytes()
            .zip(hello.token.bytes())
            .fold(0u8, |a, (b, c)| a | (b ^ c));
        if token.is_empty() || token.len() != hello.token.len() || difference != 0 {
            return Err("token");
        }
        self.token = None;
        Ok(())
    }
}
