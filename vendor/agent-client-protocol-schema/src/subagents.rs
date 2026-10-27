//! Unstable ACP Subagent session association types (RFD: Subagent Sessions).
//! Vendored into Volt's schema fork so `agent-client-protocol` 0.10 can deserialize
//! `subagent_update` / session messages without a full SDK 3.x rewrite.

use crate::{ContentBlock, Meta, SessionId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Empty object advertised as `clientCapabilities.subagents: {}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SubagentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SubagentCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Client-initiated mutations permitted for one ACP Subagent session.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SubagentSessionCapabilities {
    /// `{}` means cancel is supported; omitted/null means unsupported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SubagentSessionCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cancel_supported(&self) -> bool {
        self.cancel.is_some()
    }
}

/// Foreground work snapshot mirrored onto the parent association.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SubagentStateUpdate {
    Running {},
    Idle {
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "stopReason")]
        stop_reason: Option<String>,
    },
    RequiresAction {},
    Unknown {},
}

/// Parent announces or patches ownership of a child session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SubagentUpdate {
    pub session_id: SessionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<SubagentSessionCapabilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<SubagentStateUpdate>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SubagentUpdate {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            title: None,
            description: None,
            capabilities: None,
            state: None,
            meta: None,
        }
    }
}

/// Inter-session message upsert (enclosing transcript owns the entry).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionMessage {
    pub message_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_session_id: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_session_id: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentBlock>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

/// Streamed block of an inter-session message.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionMessageChunk {
    pub message_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_session_id: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_session_id: Option<SessionId>,
    pub content: ContentBlock,
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}
