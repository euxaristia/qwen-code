//! Session types and parameters.

use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::protocol::content::ContentBlock;
use crate::protocol::session_updates::SessionUpdate;
use crate::transport::TransportError;

/// Session errors.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session new error: {0}")]
    NewSession(String),

    #[error("Session load error: {0}")]
    LoadSession(String),

    #[error("Prompt error: {0}")]
    Prompt(#[from] TransportError),
}

/// Parameters for loading a session.
#[derive(Debug, Clone, Deserialize)]
pub struct LoadSessionParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(rename = "mcpServers", skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<Value>>,
}

/// A lightweight session handle that just tracks the session ID.
///
/// The actual prompt processing is handled directly by [`crate::AcpClient`]
/// since it owns the transport layer.
pub struct Session {
    session_id: String,
}

impl Session {
    /// Create a new Session reference.
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Cancel the current session's operation (returns the cancel message).
    pub fn cancel_message() -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancel",
        })
        .to_string()
    }

    /// Build a prompt request JSON string.
    pub fn build_prompt_request(session_id: &str, prompts: &[ContentBlock]) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": "prompt-1",
            "method": "prompt",
            "params": {
                "sessionId": session_id,
                "prompts": prompts,
            }
        })
        .to_string()
    }

    /// Parse a line from the agent and determine its type.
    ///
    /// Returns `true` if this is the end of the prompt turn.
    pub fn parse_agent_line<F>(line: &str, handler: &mut F) -> bool
    where
        F: FnMut(SessionUpdate) -> bool,
    {
        if let Ok(msg) = serde_json::from_str::<Value>(line) {
            // Check if this is a prompt response (end of turn)
            if msg
                .get("result")
                .and_then(|r| r.get("stopReason"))
                .is_some()
            {
                return true;
            }

            // Check if this is a session notification (session update)
            if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
                if method == "session/update" {
                    if let Some(params) = msg.get("params") {
                        if let Some(update) = params.get("update") {
                            match serde_json::from_value::<SessionUpdate>(update.clone()) {
                                Ok(session_update) => {
                                    return handler(session_update);
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "failed to parse session update");
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::content::TextContent;

    #[test]
    fn test_session_id() {
        let session = Session::new("test-session-123".to_string());
        assert_eq!(session.session_id(), "test-session-123");
    }

    #[test]
    fn test_load_session_params() {
        let params = LoadSessionParams {
            session_id: "test-session-123".to_string(),
            cwd: Some("/tmp".to_string()),
            mcp_servers: None,
        };
        assert_eq!(params.session_id, "test-session-123");
        assert_eq!(params.cwd.as_deref(), Some("/tmp"));
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::text("Hello, world!");
        match block {
            ContentBlock::Text(TextContent { text }) => {
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn test_content_block_from_string() {
        let block: ContentBlock = "test message".to_string().into();
        match block {
            ContentBlock::Text(TextContent { text }) => {
                assert_eq!(text, "test message");
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn test_cancel_message() {
        let msg = Session::cancel_message();
        assert!(msg.contains("notifications/cancel"));
        assert!(msg.contains("2.0"));
    }

    #[test]
    fn test_parse_end_of_turn() {
        let line = r#"{"jsonrpc":"2.0","id":"prompt-1","result":{"stopReason":"endTurn"}}"#;
        let mut updates = Vec::new();
        let ended = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(ended);
        assert!(updates.is_empty());
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "not json";
        let mut called = false;
        let ended = Session::parse_agent_line(line, &mut |_update| {
            called = true;
            false
        });
        assert!(!ended);
        assert!(!called);
    }

    #[test]
    fn test_parse_tool_call_update() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"tool_call_update","toolCallId":"tc-1","content":{"status":"running"}}}}"#;
        let mut updates = Vec::new();
        let ended = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(!ended);
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            SessionUpdate::ToolCallUpdate(tc) => {
                assert_eq!(tc.tool_call_id, "tc-1");
            }
            _ => panic!("Expected ToolCallUpdate"),
        }
    }

    #[test]
    fn test_parse_available_commands_update() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"available_commands_update","commands":[{"name":"/help"},{"name":"/clear"}]}}}"#;
        let mut updates = Vec::new();
        let ended = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(!ended);
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            SessionUpdate::AvailableCommandsUpdate(cmds) => {
                assert_eq!(cmds.commands.len(), 2);
                assert_eq!(cmds.commands[0].name, "/help");
            }
            _ => panic!("Expected AvailableCommandsUpdate"),
        }
    }

    #[test]
    fn test_parse_current_mode_update() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"current_mode_update","mode":"plan"}}}"#;
        let mut updates = Vec::new();
        let ended = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(!ended);
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            SessionUpdate::CurrentModeUpdate(mode) => {
                assert_eq!(mode.mode, "plan");
            }
            _ => panic!("Expected CurrentModeUpdate"),
        }
    }

    #[test]
    fn test_parse_plan_update() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"plan_update","plan":{"entries":[]}}}}"#;
        let mut updates = Vec::new();
        let ended = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(!ended);
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            SessionUpdate::PlanUpdate(_) => {}
            _ => panic!("Expected PlanUpdate"),
        }
    }

    #[test]
    fn test_parse_handler_returns_true_stops() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"agent_message_chunk","content":{"text":"hello"}}}}"#;
        let mut handler_called = false;
        let ended = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            true // stop processing
        });
        assert!(ended);
        assert!(handler_called);
    }

    #[test]
    fn test_parse_unknown_method_not_crash() {
        let line = r#"{"jsonrpc":"2.0","method":"unknown/method","params":{}}"#;
        let mut handler_called = false;
        let ended = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            false
        });
        assert!(!ended);
        assert!(!handler_called); // unknown method shouldn't call handler
    }

    #[test]
    fn test_parse_missing_method() {
        let line =
            r#"{"jsonrpc":"2.0","params":{"update":{"type":"agent_message_chunk","content":{}}}}"#;
        let mut handler_called = false;
        let ended = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            false
        });
        assert!(!ended);
        assert!(!handler_called);
    }

    #[test]
    fn test_parse_session_update_without_update_field() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{}}"#;
        let mut handler_called = false;
        let ended = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            false
        });
        assert!(!ended);
        assert!(!handler_called);
    }

    #[test]
    fn test_parse_agent_message_chunk_with_text() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"agent_message_chunk","content":{"text":"Hello world"}}}}"#;
        let mut text_chunks = Vec::new();
        let ended = Session::parse_agent_line(line, &mut |update| {
            if let SessionUpdate::AgentMessageChunk(chunk) = update {
                if let Some(content) = &chunk.content {
                    if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                        text_chunks.push(text.to_string());
                    }
                }
            }
            false
        });
        assert!(!ended);
        assert_eq!(text_chunks, vec!["Hello world"]);
    }

    #[test]
    fn test_parse_stop_reason_end_turn() {
        let line = r#"{"jsonrpc":"2.0","id":"prompt-1","result":{"stopReason":"end-turn"}}"#;
        let ended = Session::parse_agent_line(line, &mut |_update| false);
        assert!(ended);
    }

    #[test]
    fn test_parse_stop_reason_cancelled() {
        let line = r#"{"jsonrpc":"2.0","id":"prompt-1","result":{"stopReason":"cancelled"}}"#;
        let ended = Session::parse_agent_line(line, &mut |_update| false);
        assert!(ended);
    }

    #[test]
    fn test_build_prompt_request() {
        let prompts = vec![ContentBlock::text("Hello")];
        let json = Session::build_prompt_request("sess-1", &prompts);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["method"], "prompt");
        assert_eq!(value["params"]["sessionId"], "sess-1");
        assert_eq!(value["params"]["prompts"][0]["type"], "text");
    }

    #[test]
    fn test_cancel_message_format() {
        let msg = Session::cancel_message();
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["method"], "notifications/cancel");
        assert!(value.get("id").is_none()); // notifications have no id
    }
}
