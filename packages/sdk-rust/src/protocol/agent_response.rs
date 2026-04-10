//! Agent-to-Client response types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    pub result: InitializeResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: AgentCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    #[serde(rename = "loadSession", skip_serializing_if = "Option::is_none")]
    pub load_session: Option<bool>,
    #[serde(rename = "promptCapabilities", skip_serializing_if = "Option::is_none")]
    pub prompt_capabilities: Option<PromptCapabilities>,
    #[serde(rename = "mcpCapabilities", skip_serializing_if = "Option::is_none")]
    pub mcp_capabilities: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    #[serde(rename = "embeddedContext", skip_serializing_if = "Option::is_none")]
    pub embedded_context: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_response_deserialization() {
        let json = r#"{"result":{"protocolVersion":"1.0","capabilities":{"loadSession":true}}}"#;
        let resp: InitializeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.result.protocol_version, "1.0");
        assert_eq!(resp.result.capabilities.load_session, Some(true));
    }

    #[test]
    fn test_agent_capabilities_full() {
        let caps = AgentCapabilities {
            load_session: Some(true),
            prompt_capabilities: Some(PromptCapabilities {
                image: Some(true),
                audio: Some(false),
                embedded_context: Some(true),
            }),
            mcp_capabilities: Some(serde_json::json!({"servers": 3})),
        };
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("\"loadSession\":true"));
        assert!(json.contains("\"image\":true"));
        assert!(json.contains("\"audio\":false"));
        assert!(json.contains("\"embeddedContext\":true"));
        assert!(json.contains("\"servers\":3"));
    }

    #[test]
    fn test_agent_capabilities_null_fields() {
        let caps = AgentCapabilities {
            load_session: None,
            prompt_capabilities: None,
            mcp_capabilities: None,
        };
        let json = serde_json::to_string(&caps).unwrap();
        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn test_agent_capabilities_deserialization() {
        let json = r#"{"loadSession":false,"promptCapabilities":{"image":true,"audio":true,"embeddedContext":false}}"#;
        let caps: AgentCapabilities = serde_json::from_str(json).unwrap();
        assert_eq!(caps.load_session, Some(false));
        assert_eq!(caps.prompt_capabilities.as_ref().unwrap().image, Some(true));
        assert_eq!(caps.prompt_capabilities.as_ref().unwrap().audio, Some(true));
        assert_eq!(
            caps.prompt_capabilities.as_ref().unwrap().embedded_context,
            Some(false)
        );
    }

    #[test]
    fn test_prompt_capabilities_serialization() {
        let pc = PromptCapabilities {
            image: Some(true),
            audio: Some(true),
            embedded_context: Some(true),
        };
        let json = serde_json::to_string(&pc).unwrap();
        assert!(json.contains("\"image\":true"));
        assert!(json.contains("\"audio\":true"));
        assert!(json.contains("\"embeddedContext\":true"));
    }

    #[test]
    fn test_prompt_capabilities_null() {
        let pc = PromptCapabilities {
            image: None,
            audio: None,
            embedded_context: None,
        };
        let json = serde_json::to_string(&pc).unwrap();
        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn test_prompt_capabilities_partial() {
        let pc = PromptCapabilities {
            image: Some(true),
            audio: None,
            embedded_context: None,
        };
        let json = serde_json::to_string(&pc).unwrap();
        assert_eq!(json, r#"{"image":true}"#);
    }

    #[test]
    fn test_initialize_result_serialization() {
        let result = InitializeResult {
            protocol_version: "1.0".to_string(),
            capabilities: AgentCapabilities {
                load_session: Some(true),
                prompt_capabilities: None,
                mcp_capabilities: None,
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"protocolVersion\":\"1.0\""));
        assert!(json.contains("\"loadSession\":true"));
    }

    #[test]
    fn test_mcp_capabilities_arbitrary_json() {
        let caps = AgentCapabilities {
            load_session: None,
            prompt_capabilities: None,
            mcp_capabilities: Some(serde_json::json!({
                "tools": ["read", "write"],
                "version": "0.1"
            })),
        };
        let json = serde_json::to_string(&caps).unwrap();
        let parsed: AgentCapabilities = serde_json::from_str(&json).unwrap();
        let mcp = parsed.mcp_capabilities.unwrap();
        assert_eq!(mcp["tools"][0], "read");
        assert_eq!(mcp["version"], "0.1");
    }

    #[test]
    fn test_initialize_response_with_error() {
        // Real ACP responses might have error fields too
        let json = r#"{"result":{"protocolVersion":"1.0","capabilities":{}},"error":null}"#;
        let resp: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(resp.get("result").is_some());
    }
}
