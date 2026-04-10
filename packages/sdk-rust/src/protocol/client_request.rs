//! Client-to-Agent request types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    pub method: String,
    pub params: InitializeRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequestParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(rename = "fs", skip_serializing_if = "Option::is_none")]
    pub file_system: Option<FileSystemCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<TerminalCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCapability {
    pub enabled: bool,
}

impl Default for InitializeRequestParams {
    fn default() -> Self {
        Self {
            protocol_version: "1.0".to_string(),
            capabilities: ClientCapabilities {
                file_system: Some(FileSystemCapabilities {
                    read: Some(true),
                    write: Some(true),
                }),
                terminal: Some(TerminalCapability { enabled: true }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_request_params_default() {
        let params = InitializeRequestParams::default();
        assert_eq!(params.protocol_version, "1.0");
        assert!(params.capabilities.file_system.is_some());
        assert!(params.capabilities.terminal.is_some());
    }

    #[test]
    fn test_initialize_request_params_default_capabilities() {
        let params = InitializeRequestParams::default();
        let fs = params.capabilities.file_system.unwrap();
        assert_eq!(fs.read, Some(true));
        assert_eq!(fs.write, Some(true));
        assert!(params.capabilities.terminal.unwrap().enabled);
    }

    #[test]
    fn test_file_system_capabilities_serialization() {
        let fs = FileSystemCapabilities {
            read: Some(true),
            write: Some(false),
        };
        let json = serde_json::to_string(&fs).unwrap();
        assert!(json.contains("\"read\":true"));
        assert!(json.contains("\"write\":false"));
    }

    #[test]
    fn test_file_system_capabilities_deserialization() {
        let json = r#"{"read":true,"write":true}"#;
        let fs: FileSystemCapabilities = serde_json::from_str(json).unwrap();
        assert_eq!(fs.read, Some(true));
        assert_eq!(fs.write, Some(true));
    }

    #[test]
    fn test_terminal_capability_serialization() {
        let tc = TerminalCapability { enabled: true };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_terminal_capability_deserialization() {
        let json = r#"{"enabled":false}"#;
        let tc: TerminalCapability = serde_json::from_str(json).unwrap();
        assert!(!tc.enabled);
    }

    #[test]
    fn test_client_capabilities_full_serialization() {
        let caps = ClientCapabilities {
            file_system: Some(FileSystemCapabilities {
                read: Some(true),
                write: Some(true),
            }),
            terminal: Some(TerminalCapability { enabled: true }),
        };
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("\"fs\""));
        assert!(json.contains("\"terminal\""));
    }

    #[test]
    fn test_client_capabilities_null_fields() {
        let caps = ClientCapabilities {
            file_system: None,
            terminal: None,
        };
        let json = serde_json::to_string(&caps).unwrap();
        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn test_initialize_request_params_serialization() {
        let params = InitializeRequestParams::default();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"protocolVersion\":\"1.0\""));
        assert!(json.contains("\"fs\""));
        assert!(json.contains("\"terminal\""));
    }

    #[test]
    fn test_initialize_request_params_custom_values() {
        let params = InitializeRequestParams {
            protocol_version: "2.0".to_string(),
            capabilities: ClientCapabilities {
                file_system: Some(FileSystemCapabilities {
                    read: Some(false),
                    write: Some(false),
                }),
                terminal: Some(TerminalCapability { enabled: false }),
            },
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"protocolVersion\":\"2.0\""));
        assert!(json.contains("\"read\":false"));
        assert!(json.contains("\"enabled\":false"));
    }

    #[test]
    fn test_initialize_request_params_deserialization() {
        let json = r#"{"protocolVersion":"1.0","capabilities":{"fs":{"read":true,"write":true},"terminal":{"enabled":true}}}"#;
        let params: InitializeRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.protocol_version, "1.0");
        assert!(params.capabilities.file_system.is_some());
    }

    #[test]
    fn test_file_system_capabilities_optional_fields() {
        let fs = FileSystemCapabilities {
            read: None,
            write: None,
        };
        let json = serde_json::to_string(&fs).unwrap();
        assert_eq!(json, r#"{}"#);
    }
}
