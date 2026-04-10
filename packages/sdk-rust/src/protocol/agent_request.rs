//! Agent-to-Client request types (agent requests, client responds).

use serde::{Deserialize, Serialize};

/// Read a text file request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadTextFileRequest {
    pub method: String,
    pub params: ReadTextFileRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadTextFileRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl Default for ReadTextFileRequest {
    fn default() -> Self {
        Self {
            method: "fs/read_text_file".to_string(),
            params: ReadTextFileRequestParams {
                session_id: String::new(),
                path: String::new(),
                line: None,
                limit: None,
            },
        }
    }
}

/// Write a text file request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTextFileRequest {
    pub method: String,
    pub params: WriteTextFileRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTextFileRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub path: String,
    pub content: String,
}

/// Request permission request (agent asks user for permission).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPermissionRequest {
    pub method: String,
    pub params: RequestPermissionRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPermissionRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    pub tool: String,
    pub options: Vec<PermissionOptionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOptionItem {
    pub kind: String,
    pub label: String,
}

/// Terminal-related request types.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalRequest {
    pub method: String,
    pub params: CreateTerminalRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<super::terminal::EnvVariable>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseTerminalRequest {
    pub method: String,
    pub params: ReleaseTerminalRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseTerminalRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForTerminalExitRequest {
    pub method: String,
    pub params: WaitForTerminalExitRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForTerminalExitRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputRequest {
    pub method: String,
    pub params: TerminalOutputRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillTerminalCommandRequest {
    pub method: String,
    pub params: KillTerminalCommandRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillTerminalCommandRequestParams {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
    #[serde(rename = "commandId")]
    pub command_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_text_file_request_serialization() {
        let req = ReadTextFileRequest {
            method: "fs/read_text_file".to_string(),
            params: ReadTextFileRequestParams {
                session_id: "sess-1".to_string(),
                path: "/path/to/file.txt".to_string(),
                line: Some(10),
                limit: Some(5),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"fs/read_text_file\""));
        assert!(json.contains("\"sessionId\":\"sess-1\""));
        assert!(json.contains("\"path\":\"/path/to/file.txt\""));
        assert!(json.contains("\"line\":10"));
        assert!(json.contains("\"limit\":5"));
    }

    #[test]
    fn test_read_text_file_request_no_line_limit() {
        let req = ReadTextFileRequest {
            method: "fs/read_text_file".to_string(),
            params: ReadTextFileRequestParams {
                session_id: "sess-1".to_string(),
                path: "config.toml".to_string(),
                line: None,
                limit: None,
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("\"line\""));
        assert!(!json.contains("\"limit\""));
    }

    #[test]
    fn test_read_text_file_request_deserialization() {
        let json = r#"{"method":"fs/read_text_file","params":{"sessionId":"s1","path":"src/main.rs","line":1,"limit":100}}"#;
        let req: ReadTextFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "fs/read_text_file");
        assert_eq!(req.params.path, "src/main.rs");
        assert_eq!(req.params.line, Some(1));
    }

    #[test]
    fn test_write_text_file_request_serialization() {
        let req = WriteTextFileRequest {
            method: "fs/write_text_file".to_string(),
            params: WriteTextFileRequestParams {
                session_id: "sess-2".to_string(),
                path: "output.txt".to_string(),
                content: "Hello, World!".to_string(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"fs/write_text_file\""));
        assert!(json.contains("\"content\":\"Hello, World!\""));
    }

    #[test]
    fn test_write_text_file_request_deserialization() {
        let json = r#"{"method":"fs/write_text_file","params":{"sessionId":"s1","path":"test.txt","content":"data"}}"#;
        let req: WriteTextFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.params.content, "data");
    }

    #[test]
    fn test_request_permission_request_serialization() {
        let req = RequestPermissionRequest {
            method: "requestPermission".to_string(),
            params: RequestPermissionRequestParams {
                session_id: "sess-1".to_string(),
                tool_call_id: "tc-1".to_string(),
                tool: "ReadFile".to_string(),
                options: vec![
                    PermissionOptionItem {
                        kind: "Allow".to_string(),
                        label: "Allow".to_string(),
                    },
                    PermissionOptionItem {
                        kind: "Deny".to_string(),
                        label: "Deny".to_string(),
                    },
                ],
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"tool\":\"ReadFile\""));
        assert!(json.contains("\"options\""));
        assert_eq!(req.params.options.len(), 2);
    }

    #[test]
    fn test_request_permission_request_deserialization() {
        let json = r#"{"method":"requestPermission","params":{"sessionId":"s1","toolCallId":"tc-1","tool":"Shell","options":[{"kind":"Allow","label":"Allow"}]}}"#;
        let req: RequestPermissionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.params.tool, "Shell");
        assert_eq!(req.params.options.len(), 1);
        assert_eq!(req.params.options[0].kind, "Allow");
    }

    #[test]
    fn test_create_terminal_request_serialization() {
        use crate::protocol::terminal::EnvVariable;

        let req = CreateTerminalRequest {
            method: "terminal/create".to_string(),
            params: CreateTerminalRequestParams {
                session_id: "sess-1".to_string(),
                command: "ls -la".to_string(),
                env: Some(vec![EnvVariable {
                    key: "TERM".to_string(),
                    value: "xterm-256color".to_string(),
                }]),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"command\":\"ls -la\""));
        assert!(json.contains("\"TERM\""));
    }

    #[test]
    fn test_create_terminal_request_deserialization() {
        let json =
            r#"{"method":"terminal/create","params":{"sessionId":"s1","command":"echo hello"}}"#;
        let req: CreateTerminalRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.params.command, "echo hello");
        assert!(req.params.env.is_none());
    }

    #[test]
    fn test_release_terminal_request_serialization() {
        let req = ReleaseTerminalRequest {
            method: "terminal/release".to_string(),
            params: ReleaseTerminalRequestParams {
                session_id: "sess-1".to_string(),
                terminal_id: "term-1".to_string(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"terminalId\":\"term-1\""));
    }

    #[test]
    fn test_wait_for_terminal_exit_request_serialization() {
        let req = WaitForTerminalExitRequest {
            method: "terminal/waitForExit".to_string(),
            params: WaitForTerminalExitRequestParams {
                session_id: "sess-1".to_string(),
                terminal_id: "term-1".to_string(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"terminal/waitForExit\""));
    }

    #[test]
    fn test_terminal_output_request_serialization() {
        let req = TerminalOutputRequest {
            method: "terminal/output".to_string(),
            params: TerminalOutputRequestParams {
                session_id: "sess-1".to_string(),
                terminal_id: "term-1".to_string(),
                limit: Some(1000),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"limit\":1000"));
    }

    #[test]
    fn test_terminal_output_request_no_limit() {
        let req = TerminalOutputRequest {
            method: "terminal/output".to_string(),
            params: TerminalOutputRequestParams {
                session_id: "sess-1".to_string(),
                terminal_id: "term-1".to_string(),
                limit: None,
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("\"limit\""));
    }

    #[test]
    fn test_kill_terminal_command_request_serialization() {
        let req = KillTerminalCommandRequest {
            method: "terminal/killCommand".to_string(),
            params: KillTerminalCommandRequestParams {
                session_id: "sess-1".to_string(),
                terminal_id: "term-1".to_string(),
                command_id: "cmd-1".to_string(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"commandId\":\"cmd-1\""));
    }
}
