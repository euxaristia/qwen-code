//! ACP Client implementation — the main entry point.

use crate::protocol::agent_response::InitializeResponse;
use crate::protocol::client_request::InitializeRequestParams;
use crate::protocol::content::ContentBlock;
use crate::protocol::session_updates::SessionUpdate;
use crate::session::{LoadSessionParams, SessionError};
use crate::transport::process::{ProcessTransport, ProcessTransportOptions};
use crate::transport::{Transport, TransportError};
use serde_json::Value;
use thiserror::Error;
use tracing::debug;

/// Client initialization errors.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Agent initialization failed: {0}")]
    AgentInitialize(String),

    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),
}

/// Parameters for creating a new session.
#[derive(Debug, Clone, Default)]
pub struct NewSessionParams {
    /// Working directory for the session.
    pub cwd: Option<String>,
    /// MCP server configurations.
    pub mcp_servers: Option<Vec<Value>>,
}

/// The main ACP client.
///
/// This is the entry point for communicating with AI agents. It handles the
/// initialization handshake and provides methods for creating/loading sessions
/// and sending prompts.
pub struct AcpClient {
    transport: ProcessTransport,
    /// The most recently created/loaded session ID.
    current_session_id: Option<String>,
}

impl AcpClient {
    /// Create a new ACP client with default transport options.
    ///
    /// This starts the transport layer and performs the initialization handshake.
    pub async fn new() -> Result<Self, ClientError> {
        Self::with_options(ProcessTransportOptions::default()).await
    }

    /// Create a new ACP client with custom transport options.
    pub async fn with_options(opts: ProcessTransportOptions) -> Result<Self, ClientError> {
        let mut transport = ProcessTransport::new(opts);

        // Start transport
        transport
            .start()
            .await
            .map_err(|e| ClientError::AgentInitialize(format!("transport start failed: {}", e)))?;

        // Perform initialization handshake
        let init_params = InitializeRequestParams::default();

        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "init-1",
            "method": "initialize",
            "params": {
                "protocolVersion": init_params.protocol_version,
                "capabilities": {
                    "fs": {
                        "read": init_params.capabilities.file_system.as_ref().and_then(|f| f.read),
                        "write": init_params.capabilities.file_system.as_ref().and_then(|f| f.write),
                    },
                    "terminal": {
                        "enabled": init_params.capabilities.terminal.as_ref().map(|t| t.enabled),
                    },
                },
            }
        });

        let message = serde_json::to_string(&init_request).unwrap();
        debug!(message, "sending initialize request");

        let response = transport.request(&message).await.map_err(|e| {
            ClientError::AgentInitialize(format!("transport error during init: {}", e))
        })?;

        debug!(response, "received initialize response");

        // Parse response and check for errors
        let init_response: InitializeResponse = serde_json::from_str(&response).map_err(|e| {
            ClientError::AgentInitialize(format!("failed to parse init response: {}", e))
        })?;

        debug!(capabilities = ?init_response.result.capabilities, "agent initialized successfully");

        Ok(Self {
            transport,
            current_session_id: None,
        })
    }

    /// Create a new session with default parameters.
    pub async fn new_session(&mut self) -> Result<(), ClientError> {
        self.new_session_with(NewSessionParams::default()).await
    }

    /// Create a new session with custom parameters.
    pub async fn new_session_with(&mut self, params: NewSessionParams) -> Result<(), ClientError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "new-session-1",
            "method": "newSession",
            "params": {
                "cwd": params.cwd,
                "mcpServers": params.mcp_servers,
            }
        });

        let message = serde_json::to_string(&request).unwrap();
        debug!(message, "sending newSession request");

        let response = self.transport.request(&message).await.map_err(|e| {
            ClientError::Session(SessionError::NewSession(format!("transport error: {}", e)))
        })?;

        debug!(response, "newSession response");

        // Parse the response to get session ID
        let response_value: Value = serde_json::from_str(&response).map_err(|e| {
            ClientError::Session(SessionError::NewSession(format!(
                "failed to parse response: {}",
                e
            )))
        })?;

        if let Some(error) = response_value.get("error") {
            return Err(ClientError::Session(SessionError::NewSession(format!(
                "agent error: {}",
                error
            ))));
        }

        let session_id = response_value
            .get("result")
            .and_then(|r| r.get("sessionId"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| {
                ClientError::Session(SessionError::NewSession(
                    "no sessionId in response".to_string(),
                ))
            })?
            .to_string();

        self.current_session_id = Some(session_id);
        Ok(())
    }

    /// Load an existing session.
    pub async fn load_session(&mut self, params: &LoadSessionParams) -> Result<(), ClientError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "load-session-1",
            "method": "loadSession",
            "params": {
                "sessionId": params.session_id,
                "cwd": params.cwd,
                "mcpServers": params.mcp_servers,
            }
        });

        let message = serde_json::to_string(&request).unwrap();
        debug!(message, "sending loadSession request");

        self.transport.request(&message).await.map_err(|e| {
            ClientError::Session(SessionError::LoadSession(format!("transport error: {}", e)))
        })?;

        self.current_session_id = Some(params.session_id.clone());
        Ok(())
    }

    /// Get the current session ID.
    pub fn session_id(&self) -> Option<&str> {
        self.current_session_id.as_deref()
    }

    /// Send a prompt and process events via the provided handler.
    ///
    /// Requires a session to have been created via [`Self::new_session`] or [`Self::load_session`].
    pub async fn send_prompt<F>(
        &mut self,
        prompts: &[ContentBlock],
        mut handler: F,
    ) -> Result<(), SessionError>
    where
        F: FnMut(SessionUpdate) -> bool,
    {
        let session_id = self
            .current_session_id
            .as_ref()
            .ok_or(SessionError::Prompt(TransportError::NotStarted))?
            .clone();

        let prompt_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "prompt-1",
            "method": "prompt",
            "params": {
                "sessionId": session_id,
                "prompts": prompts,
            }
        });
        let message = serde_json::to_string(&prompt_request).unwrap();
        debug!(message, "send_prompt to agent");

        // Send the prompt
        self.transport
            .send(&message)
            .await
            .map_err(SessionError::Prompt)?;

        // Process the stream inline
        loop {
            let line = self
                .transport
                .request_inner()
                .await
                .map_err(SessionError::Prompt)?;

            debug!(line, "received_message from agent");

            if let Ok(msg) = serde_json::from_str::<Value>(&line) {
                // Check if this is a prompt response (end of turn)
                if msg
                    .get("result")
                    .and_then(|r| r.get("stopReason"))
                    .is_some()
                {
                    debug!("rcv prompt_turn_end");
                    return Ok(());
                }

                // Check if this is a session notification (session update)
                if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
                    if method == "session/update" {
                        if let Some(params) = msg.get("params") {
                            if let Some(update) = params.get("update") {
                                match serde_json::from_value::<SessionUpdate>(update.clone()) {
                                    Ok(session_update) => {
                                        if handler(session_update) {
                                            return Ok(());
                                        }
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
        }
    }

    /// Send a prompt and collect all text content blocks into a Vec<String>.
    pub async fn send_prompt_text(
        &mut self,
        prompts: &[ContentBlock],
    ) -> Result<Vec<String>, SessionError> {
        let mut results = Vec::new();

        self.send_prompt(prompts, |update| {
            if let SessionUpdate::AgentMessageChunk(chunk) = update {
                if let Some(content) = chunk.content {
                    if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                        results.push(text.to_string());
                    }
                }
            }
            false
        })
        .await?;

        Ok(results)
    }

    /// Cancel the current operation.
    pub async fn cancel(&mut self) -> Result<(), TransportError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancel",
        });
        let message = serde_json::to_string(&notification).unwrap();
        debug!(message, "sending cancel notification");
        self.transport.send(&message).await
    }

    /// Close the client and release resources.
    pub async fn close(&mut self) -> Result<(), TransportError> {
        self.transport.close().await
    }
}
