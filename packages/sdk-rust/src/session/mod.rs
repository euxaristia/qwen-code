//! Session types and parameters.

pub mod event_consumer;

use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::protocol::content::ContentBlock;
use crate::protocol::session_updates::SessionUpdate;
use crate::protocol::{
    CreateTerminalRequestParams, KillTerminalCommandRequestParams, ReadTextFileRequestParams,
    ReleaseTerminalRequestParams, RequestPermissionRequestParams, TerminalOutputRequestParams,
    WaitForTerminalExitRequestParams, WriteTextFileRequestParams,
};
use crate::session::event_consumer::AgentEventConsumer;
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

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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
    /// Returns `Some(AgentRequest)` if this is an agent-to-client request that needs handling.
    /// Returns `true` via the handler if this is the end of the prompt turn.
    /// Returns `false` if this is a session update or unrecognized.
    pub fn parse_agent_line<F>(line: &str, handler: &mut F) -> ParseResult
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
                return ParseResult::EndOfTurn;
            }

            // Check if this is a session notification (session update)
            if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
                if method == "session/update" {
                    if let Some(params) = msg.get("params") {
                        if let Some(update) = params.get("update") {
                            match serde_json::from_value::<SessionUpdate>(update.clone()) {
                                Ok(session_update) => {
                                    return if handler(session_update) {
                                        ParseResult::HandlerStop
                                    } else {
                                        ParseResult::Continue
                                    };
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "failed to parse session update");
                                }
                            }
                        }
                    }
                }

                // Check for agent-to-client requests
                if let Some(request) = Self::parse_agent_request(method, &msg) {
                    return ParseResult::AgentRequest(request);
                }
            }
        }
        ParseResult::Continue
    }

    /// Parse an agent-to-client request from a message.
    fn parse_agent_request(method: &str, msg: &Value) -> Option<AgentRequest> {
        let params = msg.get("params")?;
        let id = msg.get("id").and_then(|v| v.as_str()).map(String::from);

        match method {
            "fs/read_text_file" => {
                if let Ok(p) = serde_json::from_value::<ReadTextFileRequestParams>(params.clone()) {
                    return Some(AgentRequest::ReadTextFile { id, params: p });
                }
            }
            "fs/write_text_file" => {
                if let Ok(p) = serde_json::from_value::<WriteTextFileRequestParams>(params.clone())
                {
                    return Some(AgentRequest::WriteTextFile { id, params: p });
                }
            }
            "requestPermission" => {
                if let Ok(p) =
                    serde_json::from_value::<RequestPermissionRequestParams>(params.clone())
                {
                    return Some(AgentRequest::RequestPermission { id, params: p });
                }
            }
            "terminal/create" => {
                if let Ok(p) = serde_json::from_value::<CreateTerminalRequestParams>(params.clone())
                {
                    return Some(AgentRequest::CreateTerminal { id, params: p });
                }
            }
            "terminal/release" => {
                if let Ok(p) =
                    serde_json::from_value::<ReleaseTerminalRequestParams>(params.clone())
                {
                    return Some(AgentRequest::ReleaseTerminal { id, params: p });
                }
            }
            "terminal/waitForExit" => {
                if let Ok(p) =
                    serde_json::from_value::<WaitForTerminalExitRequestParams>(params.clone())
                {
                    return Some(AgentRequest::WaitForTerminalExit { id, params: p });
                }
            }
            "terminal/output" => {
                if let Ok(p) = serde_json::from_value::<TerminalOutputRequestParams>(params.clone())
                {
                    return Some(AgentRequest::TerminalOutput { id, params: p });
                }
            }
            "terminal/killCommand" => {
                if let Ok(p) =
                    serde_json::from_value::<KillTerminalCommandRequestParams>(params.clone())
                {
                    return Some(AgentRequest::KillTerminalCommand { id, params: p });
                }
            }
            _ => {}
        }
        None
    }

    /// Build a JSON-RPC response for an agent request.
    pub fn build_response<R: serde::Serialize>(id: &str, result: R) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        })
        .to_string()
    }

    /// Build a JSON-RPC error response.
    pub fn build_error_response(id: &str, code: i64, message: &str) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message,
            },
        })
        .to_string()
    }
}

/// Result of parsing an agent line.
#[derive(Debug)]
pub enum ParseResult {
    /// End of prompt turn (stop reading).
    EndOfTurn,
    /// Handler returned true (stop reading).
    HandlerStop,
    /// Continue reading more lines.
    Continue,
    /// Agent-to-client request that needs handling.
    AgentRequest(AgentRequest),
}

/// Agent-to-client request types.
#[derive(Debug)]
pub enum AgentRequest {
    ReadTextFile {
        id: Option<String>,
        params: ReadTextFileRequestParams,
    },
    WriteTextFile {
        id: Option<String>,
        params: WriteTextFileRequestParams,
    },
    RequestPermission {
        id: Option<String>,
        params: RequestPermissionRequestParams,
    },
    CreateTerminal {
        id: Option<String>,
        params: CreateTerminalRequestParams,
    },
    ReleaseTerminal {
        id: Option<String>,
        params: ReleaseTerminalRequestParams,
    },
    WaitForTerminalExit {
        id: Option<String>,
        params: WaitForTerminalExitRequestParams,
    },
    TerminalOutput {
        id: Option<String>,
        params: TerminalOutputRequestParams,
    },
    KillTerminalCommand {
        id: Option<String>,
        params: KillTerminalCommandRequestParams,
    },
}

impl AgentRequest {
    /// Get the request ID for building the response.
    pub fn id(&self) -> Option<&str> {
        match self {
            AgentRequest::ReadTextFile { id, .. } => id.as_deref(),
            AgentRequest::WriteTextFile { id, .. } => id.as_deref(),
            AgentRequest::RequestPermission { id, .. } => id.as_deref(),
            AgentRequest::CreateTerminal { id, .. } => id.as_deref(),
            AgentRequest::ReleaseTerminal { id, .. } => id.as_deref(),
            AgentRequest::WaitForTerminalExit { id, .. } => id.as_deref(),
            AgentRequest::TerminalOutput { id, .. } => id.as_deref(),
            AgentRequest::KillTerminalCommand { id, .. } => id.as_deref(),
        }
    }

    /// Handle this request using the event consumer.
    pub async fn handle(
        &self,
        consumer: &dyn AgentEventConsumer,
    ) -> Result<serde_json::Value, SessionError> {
        match self {
            AgentRequest::ReadTextFile { params, .. } => {
                if let Some(fc) = consumer.file_consumer() {
                    let result = fc.on_read_text_file(params)?;
                    Ok(serde_json::to_value(result)?)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No file consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::WriteTextFile { params, .. } => {
                if let Some(fc) = consumer.file_consumer() {
                    fc.on_write_text_file(params)?;
                    Ok(serde_json::Value::Null)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No file consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::RequestPermission { params, .. } => {
                if let Some(pc) = consumer.permission_consumer() {
                    let result = pc.on_request_permission(params)?;
                    Ok(serde_json::to_value(result)?)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No permission consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::CreateTerminal { params, .. } => {
                if let Some(tc) = consumer.terminal_consumer() {
                    let result = tc.on_create_terminal(params)?;
                    Ok(serde_json::to_value(result)?)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No terminal consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::ReleaseTerminal { params, .. } => {
                if let Some(tc) = consumer.terminal_consumer() {
                    tc.on_release_terminal(params)?;
                    Ok(serde_json::Value::Null)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No terminal consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::WaitForTerminalExit { params, .. } => {
                if let Some(tc) = consumer.terminal_consumer() {
                    let result = tc.on_wait_for_terminal_exit(params)?;
                    Ok(serde_json::to_value(result)?)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No terminal consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::TerminalOutput { params, .. } => {
                if let Some(tc) = consumer.terminal_consumer() {
                    let result = tc.on_terminal_output(params)?;
                    Ok(serde_json::to_value(result)?)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No terminal consumer configured".to_string(),
                    )))
                }
            }
            AgentRequest::KillTerminalCommand { params, .. } => {
                if let Some(tc) = consumer.terminal_consumer() {
                    tc.on_kill_terminal_command(params)?;
                    Ok(serde_json::Value::Null)
                } else {
                    Err(SessionError::Prompt(TransportError::Protocol(
                        "No terminal consumer configured".to_string(),
                    )))
                }
            }
        }
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
        let result = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(matches!(result, ParseResult::EndOfTurn));
        assert!(updates.is_empty());
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "not json";
        let mut called = false;
        let result = Session::parse_agent_line(line, &mut |_update| {
            called = true;
            false
        });
        assert!(matches!(result, ParseResult::Continue));
        assert!(!called);
    }

    #[test]
    fn test_parse_tool_call_update() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"tool_call_update","toolCallId":"tc-1","content":{"status":"running"}}}}"#;
        let mut updates = Vec::new();
        let result = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(matches!(result, ParseResult::Continue));
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
        let result = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(matches!(result, ParseResult::Continue));
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
        let result = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(matches!(result, ParseResult::Continue));
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
        let result = Session::parse_agent_line(line, &mut |update| {
            updates.push(update);
            false
        });
        assert!(matches!(result, ParseResult::Continue));
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
        let result = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            true // stop processing
        });
        assert!(matches!(result, ParseResult::HandlerStop));
        assert!(handler_called);
    }

    #[test]
    fn test_parse_unknown_method_not_crash() {
        let line = r#"{"jsonrpc":"2.0","method":"unknown/method","params":{}}"#;
        let mut handler_called = false;
        let result = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            false
        });
        assert!(matches!(result, ParseResult::Continue));
        assert!(!handler_called); // unknown method shouldn't call handler
    }

    #[test]
    fn test_parse_missing_method() {
        let line =
            r#"{"jsonrpc":"2.0","params":{"update":{"type":"agent_message_chunk","content":{}}}}"#;
        let mut handler_called = false;
        let result = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            false
        });
        assert!(matches!(result, ParseResult::Continue));
        assert!(!handler_called);
    }

    #[test]
    fn test_parse_session_update_without_update_field() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{}}"#;
        let mut handler_called = false;
        let result = Session::parse_agent_line(line, &mut |_update| {
            handler_called = true;
            false
        });
        assert!(matches!(result, ParseResult::Continue));
        assert!(!handler_called);
    }

    #[test]
    fn test_parse_agent_message_chunk_with_text() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"agent_message_chunk","content":{"text":"Hello world"}}}}"#;
        let mut text_chunks = Vec::new();
        let result = Session::parse_agent_line(line, &mut |update| {
            if let SessionUpdate::AgentMessageChunk(chunk) = update {
                if let Some(content) = &chunk.content {
                    if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                        text_chunks.push(text.to_string());
                    }
                }
            }
            false
        });
        assert!(matches!(result, ParseResult::Continue));
        assert_eq!(text_chunks, vec!["Hello world"]);
    }

    #[test]
    fn test_parse_stop_reason_end_turn() {
        let line = r#"{"jsonrpc":"2.0","id":"prompt-1","result":{"stopReason":"end-turn"}}"#;
        let result = Session::parse_agent_line(line, &mut |_update| false);
        assert!(matches!(result, ParseResult::EndOfTurn));
    }

    #[test]
    fn test_parse_stop_reason_cancelled() {
        let line = r#"{"jsonrpc":"2.0","id":"prompt-1","result":{"stopReason":"cancelled"}}"#;
        let result = Session::parse_agent_line(line, &mut |_update| false);
        assert!(matches!(result, ParseResult::EndOfTurn));
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

    #[test]
    fn test_parse_read_text_file_request() {
        let line = r#"{"jsonrpc":"2.0","id":"req-1","method":"fs/read_text_file","params":{"sessionId":"s1","path":"src/main.rs","line":10,"limit":5}}"#;
        let result = Session::parse_agent_line(line, &mut |_| false);
        match result {
            ParseResult::AgentRequest(AgentRequest::ReadTextFile { id, params }) => {
                assert_eq!(id, Some("req-1".to_string()));
                assert_eq!(params.path, "src/main.rs");
                assert_eq!(params.line, Some(10));
            }
            _ => panic!("Expected ReadTextFile request, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_write_text_file_request() {
        let line = r#"{"jsonrpc":"2.0","id":"req-2","method":"fs/write_text_file","params":{"sessionId":"s1","path":"out.txt","content":"hello"}}"#;
        let result = Session::parse_agent_line(line, &mut |_| false);
        match result {
            ParseResult::AgentRequest(AgentRequest::WriteTextFile { id, params }) => {
                assert_eq!(id, Some("req-2".to_string()));
                assert_eq!(params.content, "hello");
            }
            _ => panic!("Expected WriteTextFile request"),
        }
    }

    #[test]
    fn test_parse_permission_request() {
        let line = r#"{"jsonrpc":"2.0","id":"req-3","method":"requestPermission","params":{"sessionId":"s1","toolCallId":"tc-1","tool":"ReadFile","options":[]}}"#;
        let result = Session::parse_agent_line(line, &mut |_| false);
        match result {
            ParseResult::AgentRequest(AgentRequest::RequestPermission { id, params }) => {
                assert_eq!(id, Some("req-3".to_string()));
                assert_eq!(params.tool, "ReadFile");
            }
            _ => panic!("Expected RequestPermission request"),
        }
    }

    #[test]
    fn test_parse_terminal_create_request() {
        let line = r#"{"jsonrpc":"2.0","id":"req-4","method":"terminal/create","params":{"sessionId":"s1","command":"ls"}}"#;
        let result = Session::parse_agent_line(line, &mut |_| false);
        match result {
            ParseResult::AgentRequest(AgentRequest::CreateTerminal { id, params }) => {
                assert_eq!(id, Some("req-4".to_string()));
                assert_eq!(params.command, "ls");
            }
            _ => panic!("Expected CreateTerminal request"),
        }
    }

    #[test]
    fn test_build_response() {
        let json = Session::build_response("req-1", serde_json::json!({"content": "hello"}));
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["id"], "req-1");
        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["result"]["content"], "hello");
    }

    #[test]
    fn test_build_error_response() {
        let json = Session::build_error_response("req-1", -32603, "Internal error");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["id"], "req-1");
        assert_eq!(value["error"]["code"], -32603);
        assert_eq!(value["error"]["message"], "Internal error");
    }

    #[test]
    fn test_agent_request_id() {
        let req = AgentRequest::ReadTextFile {
            id: Some("test-id".to_string()),
            params: ReadTextFileRequestParams {
                session_id: "s1".to_string(),
                path: "f.txt".to_string(),
                line: None,
                limit: None,
            },
        };
        assert_eq!(req.id(), Some("test-id"));
    }

    #[test]
    fn test_agent_request_no_id() {
        let req = AgentRequest::CreateTerminal {
            id: None,
            params: CreateTerminalRequestParams {
                session_id: "s1".to_string(),
                command: "ls".to_string(),
                env: None,
            },
        };
        assert!(req.id().is_none());
    }
}
