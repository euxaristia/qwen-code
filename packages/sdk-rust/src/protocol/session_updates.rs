//! Session update types (agent-to-client notifications).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionUpdate {
    AgentMessageChunk(AgentMessageChunk),
    ToolCallUpdate(ToolCallUpdate),
    ToolCall(ToolCallSessionUpdate),
    AvailableCommandsUpdate(AvailableCommandsUpdate),
    CurrentModeUpdate(CurrentModeUpdate),
    PlanUpdate(PlanSessionUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageChunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallUpdate {
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallSessionUpdate {
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableCommand {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableCommandsUpdate {
    pub commands: Vec<AvailableCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentModeUpdate {
    pub mode: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSessionUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_update_agent_message_chunk() {
        let json = r#"{"type":"agent_message_chunk","content":{"text":"Hello"}}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::AgentMessageChunk(chunk) => {
                assert!(chunk.content.is_some());
            }
            _ => panic!("Expected AgentMessageChunk"),
        }
    }

    #[test]
    fn test_session_update_tool_call_update() {
        let json =
            r#"{"type":"tool_call_update","toolCallId":"tc-1","content":{"status":"running"}}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::ToolCallUpdate(tc) => {
                assert_eq!(tc.tool_call_id, "tc-1");
            }
            _ => panic!("Expected ToolCallUpdate"),
        }
    }

    #[test]
    fn test_session_update_tool_call() {
        let json = r#"{"type":"tool_call","toolCallId":"tc-2","status":"pending"}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::ToolCall(tc) => {
                assert_eq!(tc.tool_call_id, "tc-2");
            }
            _ => panic!("Expected ToolCall"),
        }
    }

    #[test]
    fn test_session_update_available_commands() {
        let json = r#"{"type":"available_commands_update","commands":[{"name":"/help","description":"Get help"}]}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::AvailableCommandsUpdate(cmds) => {
                assert_eq!(cmds.commands.len(), 1);
                assert_eq!(cmds.commands[0].name, "/help");
                assert_eq!(cmds.commands[0].description, Some("Get help".to_string()));
            }
            _ => panic!("Expected AvailableCommandsUpdate"),
        }
    }

    #[test]
    fn test_session_update_current_mode() {
        let json = r#"{"type":"current_mode_update","mode":"plan"}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::CurrentModeUpdate(mode) => {
                assert_eq!(mode.mode, "plan");
            }
            _ => panic!("Expected CurrentModeUpdate"),
        }
    }

    #[test]
    fn test_session_update_plan() {
        let json =
            r#"{"type":"plan_update","plan":{"entries":[{"task":"step1","status":"done"}]}}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::PlanUpdate(plan_update) => {
                assert!(plan_update.plan.is_some());
            }
            _ => panic!("Expected PlanUpdate"),
        }
    }

    #[test]
    fn test_session_update_plan_null() {
        let json = r#"{"type":"plan_update"}"#;
        let update: SessionUpdate = serde_json::from_str(json).unwrap();
        match update {
            SessionUpdate::PlanUpdate(plan_update) => {
                assert!(plan_update.plan.is_none());
            }
            _ => panic!("Expected PlanUpdate"),
        }
    }

    #[test]
    fn test_available_command_without_description() {
        let cmd = AvailableCommand {
            name: "/clear".to_string(),
            description: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(!json.contains("\"description\""));
    }

    #[test]
    fn test_agent_message_chunk_null_content() {
        let chunk = AgentMessageChunk { content: None };
        let json = serde_json::to_string(&chunk).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_tool_call_update_no_content() {
        let tc = ToolCallUpdate {
            tool_call_id: "tc-3".to_string(),
            content: None,
        };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(!json.contains("\"content\""));
    }

    #[test]
    fn test_tool_call_session_update() {
        let tc = ToolCallSessionUpdate {
            tool_call_id: "tc-4".to_string(),
            status: Some(serde_json::json!("completed")),
        };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("\"toolCallId\":\"tc-4\""));
    }

    #[test]
    fn test_tool_call_session_update_no_status() {
        let tc = ToolCallSessionUpdate {
            tool_call_id: "tc-5".to_string(),
            status: None,
        };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(!json.contains("\"status\""));
    }

    #[test]
    fn test_empty_commands_update() {
        let update = AvailableCommandsUpdate { commands: vec![] };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"commands\":[]"));
    }

    #[test]
    fn test_plan_session_update_empty() {
        let update = PlanSessionUpdate { plan: None };
        let json = serde_json::to_string(&update).unwrap();
        assert_eq!(json, "{}");
    }
}
