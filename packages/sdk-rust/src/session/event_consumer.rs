//! Event consumer system for handling agent-to-client requests during prompts.
//!
//! This module mirrors the Java SDK's `AgentEventConsumer` hierarchy:
//! - `ContentEventConsumer` — handles streaming session updates
//! - `FileEventConsumer` — handles file read/write requests
//! - `TerminalEventConsumer` — handles terminal lifecycle requests
//! - `PermissionEventConsumer` — handles permission requests
//! - `PromptEndEventConsumer` — handles end-of-prompt events

use crate::protocol::{
    CreateTerminalRequestParams, KillTerminalCommandRequestParams, ReadTextFileRequestParams,
    ReleaseTerminalRequestParams, RequestPermissionRequestParams, TerminalOutputRequestParams,
    WaitForTerminalExitRequestParams, WriteTextFileRequestParams,
};
use crate::session::SessionError;

/// File read result from client to agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileReadResult {
    pub content: String,
    #[serde(rename = "totalLines")]
    pub total_lines: Option<i64>,
}

/// Terminal operation result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalResult {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,
}

/// Permission response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionResult {
    pub outcome: PermissionOutcome,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PermissionOutcome {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "allowAlways")]
    AllowAlways,
    #[serde(rename = "denyAlways")]
    DenyAlways,
}

/// Trait for handling file system requests from the agent.
pub trait FileEventConsumer: Send {
    /// Handle a file read request from the agent.
    fn on_read_text_file(
        &self,
        request: &ReadTextFileRequestParams,
    ) -> Result<FileReadResult, SessionError>;

    /// Handle a file write request from the agent.
    fn on_write_text_file(&self, request: &WriteTextFileRequestParams) -> Result<(), SessionError>;

    /// Timeout for file operations (default: 30 seconds).
    fn timeout_secs(&self) -> u64 {
        30
    }
}

/// Trait for handling terminal requests from the agent.
pub trait TerminalEventConsumer: Send {
    /// Handle a terminal creation request.
    fn on_create_terminal(
        &self,
        request: &CreateTerminalRequestParams,
    ) -> Result<TerminalResult, SessionError>;

    /// Handle a terminal release request.
    fn on_release_terminal(
        &self,
        request: &ReleaseTerminalRequestParams,
    ) -> Result<(), SessionError>;

    /// Handle a wait-for-terminal-exit request.
    fn on_wait_for_terminal_exit(
        &self,
        request: &WaitForTerminalExitRequestParams,
    ) -> Result<TerminalResult, SessionError>;

    /// Handle a terminal output request.
    fn on_terminal_output(
        &self,
        request: &TerminalOutputRequestParams,
    ) -> Result<TerminalResult, SessionError>;

    /// Handle a kill terminal command request.
    fn on_kill_terminal_command(
        &self,
        request: &KillTerminalCommandRequestParams,
    ) -> Result<(), SessionError>;

    /// Timeout for terminal operations (default: 60 seconds).
    fn timeout_secs(&self) -> u64 {
        60
    }
}

/// Trait for handling permission requests from the agent.
pub trait PermissionEventConsumer: Send {
    /// Handle a permission request from the agent.
    fn on_request_permission(
        &self,
        request: &RequestPermissionRequestParams,
    ) -> Result<PermissionResult, SessionError>;

    /// Timeout for permission requests (default: 300 seconds / 5 minutes).
    fn timeout_secs(&self) -> u64 {
        300
    }
}

/// Combined event consumer that handles all agent-to-client requests.
///
/// This is the primary interface used during prompt processing.
pub trait AgentEventConsumer: Send {
    fn file_consumer(&self) -> Option<&dyn FileEventConsumer>;
    fn terminal_consumer(&self) -> Option<&dyn TerminalEventConsumer>;
    fn permission_consumer(&self) -> Option<&dyn PermissionEventConsumer>;
}

/// Default implementation that delegates to sub-consumers.
pub struct DefaultAgentEventConsumer<F, T, P> {
    file: Option<F>,
    terminal: Option<T>,
    permission: Option<P>,
}

impl<F, T, P> DefaultAgentEventConsumer<F, T, P>
where
    F: FileEventConsumer,
    T: TerminalEventConsumer,
    P: PermissionEventConsumer,
{
    pub fn new(file: Option<F>, terminal: Option<T>, permission: Option<P>) -> Self {
        Self {
            file,
            terminal,
            permission,
        }
    }
}

impl<F, T, P> AgentEventConsumer for DefaultAgentEventConsumer<F, T, P>
where
    F: FileEventConsumer + 'static,
    T: TerminalEventConsumer + 'static,
    P: PermissionEventConsumer + 'static,
{
    fn file_consumer(&self) -> Option<&dyn FileEventConsumer> {
        self.file.as_ref().map(|f| f as &dyn FileEventConsumer)
    }

    fn terminal_consumer(&self) -> Option<&dyn TerminalEventConsumer> {
        self.terminal
            .as_ref()
            .map(|t| t as &dyn TerminalEventConsumer)
    }

    fn permission_consumer(&self) -> Option<&dyn PermissionEventConsumer> {
        self.permission
            .as_ref()
            .map(|p| p as &dyn PermissionEventConsumer)
    }
}

/// Simple file consumer that returns empty content (for testing).
pub struct NoOpFileConsumer;

impl FileEventConsumer for NoOpFileConsumer {
    fn on_read_text_file(
        &self,
        _request: &ReadTextFileRequestParams,
    ) -> Result<FileReadResult, SessionError> {
        Ok(FileReadResult {
            content: String::new(),
            total_lines: Some(0),
        })
    }

    fn on_write_text_file(
        &self,
        _request: &WriteTextFileRequestParams,
    ) -> Result<(), SessionError> {
        Ok(())
    }
}

/// Simple terminal consumer (for testing).
pub struct NoOpTerminalConsumer;

impl TerminalEventConsumer for NoOpTerminalConsumer {
    fn on_create_terminal(
        &self,
        _request: &CreateTerminalRequestParams,
    ) -> Result<TerminalResult, SessionError> {
        Ok(TerminalResult {
            terminal_id: "term-0".to_string(),
            output: None,
            exit_code: None,
        })
    }

    fn on_release_terminal(
        &self,
        _request: &ReleaseTerminalRequestParams,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn on_wait_for_terminal_exit(
        &self,
        _request: &WaitForTerminalExitRequestParams,
    ) -> Result<TerminalResult, SessionError> {
        Ok(TerminalResult {
            terminal_id: "term-0".to_string(),
            output: None,
            exit_code: Some(0),
        })
    }

    fn on_terminal_output(
        &self,
        _request: &TerminalOutputRequestParams,
    ) -> Result<TerminalResult, SessionError> {
        Ok(TerminalResult {
            terminal_id: "term-0".to_string(),
            output: Some(String::new()),
            exit_code: None,
        })
    }

    fn on_kill_terminal_command(
        &self,
        _request: &KillTerminalCommandRequestParams,
    ) -> Result<(), SessionError> {
        Ok(())
    }
}

/// Simple permission consumer that always allows (for testing).
pub struct AllowAllPermissionConsumer;

impl PermissionEventConsumer for AllowAllPermissionConsumer {
    fn on_request_permission(
        &self,
        _request: &RequestPermissionRequestParams,
    ) -> Result<PermissionResult, SessionError> {
        Ok(PermissionResult {
            outcome: PermissionOutcome::Allow,
        })
    }
}

/// Build a default agent event consumer for testing.
pub fn default_event_consumer(
) -> DefaultAgentEventConsumer<NoOpFileConsumer, NoOpTerminalConsumer, AllowAllPermissionConsumer> {
    DefaultAgentEventConsumer::new(
        Some(NoOpFileConsumer),
        Some(NoOpTerminalConsumer),
        Some(AllowAllPermissionConsumer),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_read_result_serialization() {
        let result = FileReadResult {
            content: "line1\nline2".to_string(),
            total_lines: Some(2),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"content\""));
        assert!(json.contains("\"totalLines\":2"));
    }

    #[test]
    fn test_file_read_result_deserialization() {
        let json = r#"{"content":"hello","totalLines":1}"#;
        let result: FileReadResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.content, "hello");
        assert_eq!(result.total_lines, Some(1));
    }

    #[test]
    fn test_terminal_result_serialization() {
        let result = TerminalResult {
            terminal_id: "term-1".to_string(),
            output: Some("output".to_string()),
            exit_code: Some(0),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"terminalId\":\"term-1\""));
        assert!(json.contains("\"exit_code\":0"));
    }

    #[test]
    fn test_terminal_result_skip_none() {
        let result = TerminalResult {
            terminal_id: "term-1".to_string(),
            output: None,
            exit_code: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("\"output\""));
        assert!(!json.contains("\"exit_code\""));
    }

    #[test]
    fn test_permission_result_allow() {
        let result = PermissionResult {
            outcome: PermissionOutcome::Allow,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"allow\""));
    }

    #[test]
    fn test_permission_result_deny() {
        let result = PermissionResult {
            outcome: PermissionOutcome::Deny,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"deny\""));
    }

    #[test]
    fn test_permission_result_allow_always() {
        let result = PermissionResult {
            outcome: PermissionOutcome::AllowAlways,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"allowAlways\""));
    }

    #[test]
    fn test_permission_result_deny_always() {
        let result = PermissionResult {
            outcome: PermissionOutcome::DenyAlways,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"denyAlways\""));
    }

    #[test]
    fn test_noop_file_consumer_read() {
        let consumer = NoOpFileConsumer;
        let request = ReadTextFileRequestParams {
            session_id: "s1".to_string(),
            path: "test.txt".to_string(),
            line: None,
            limit: None,
        };
        let result = consumer.on_read_text_file(&request).unwrap();
        assert_eq!(result.content, "");
        assert_eq!(result.total_lines, Some(0));
    }

    #[test]
    fn test_noop_file_consumer_write() {
        let consumer = NoOpFileConsumer;
        consumer
            .on_write_text_file(&WriteTextFileRequestParams {
                session_id: "s1".to_string(),
                path: "test.txt".to_string(),
                content: "data".to_string(),
            })
            .unwrap();
    }

    #[test]
    fn test_noop_terminal_consumer_create() {
        let consumer = NoOpTerminalConsumer;
        let result = consumer
            .on_create_terminal(&CreateTerminalRequestParams {
                session_id: "s1".to_string(),
                command: "ls".to_string(),
                env: None,
            })
            .unwrap();
        assert_eq!(result.terminal_id, "term-0");
    }

    #[test]
    fn test_allow_all_permission_consumer() {
        let consumer = AllowAllPermissionConsumer;
        let result = consumer
            .on_request_permission(&RequestPermissionRequestParams {
                session_id: "s1".to_string(),
                tool_call_id: "tc-1".to_string(),
                tool: "ReadFile".to_string(),
                options: vec![],
            })
            .unwrap();
        assert!(matches!(result.outcome, PermissionOutcome::Allow));
    }

    #[test]
    fn test_default_event_consumer_build() {
        let consumer = default_event_consumer();
        assert!(consumer.file_consumer().is_some());
        assert!(consumer.terminal_consumer().is_some());
        assert!(consumer.permission_consumer().is_some());
    }

    #[test]
    fn test_permission_outcome_roundtrip() {
        let outcomes = [
            PermissionOutcome::Allow,
            PermissionOutcome::Deny,
            PermissionOutcome::AllowAlways,
            PermissionOutcome::DenyAlways,
        ];
        for outcome in outcomes {
            let result = PermissionResult {
                outcome: outcome.clone(),
            };
            let json = serde_json::to_string(&result).unwrap();
            let parsed: PermissionResult = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.outcome, outcome);
        }
    }
}
