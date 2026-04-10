//! Tool domain types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ToolKind {
    ReadFile,
    EditFile,
    Shell,
    WebFetch,
    Glob,
    Grep,
    Ls,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCallStatus {
    Pending,
    InProgress,
    Success,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallUpdate {
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    pub status: ToolCallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallLocation {
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_kind_serialize_all() {
        assert_eq!(
            serde_json::to_string(&ToolKind::ReadFile).unwrap(),
            r#""read-file""#
        );
        assert_eq!(
            serde_json::to_string(&ToolKind::EditFile).unwrap(),
            r#""edit-file""#
        );
        assert_eq!(
            serde_json::to_string(&ToolKind::Shell).unwrap(),
            r#""shell""#
        );
        assert_eq!(
            serde_json::to_string(&ToolKind::WebFetch).unwrap(),
            r#""web-fetch""#
        );
        assert_eq!(serde_json::to_string(&ToolKind::Glob).unwrap(), r#""glob""#);
        assert_eq!(serde_json::to_string(&ToolKind::Grep).unwrap(), r#""grep""#);
        assert_eq!(serde_json::to_string(&ToolKind::Ls).unwrap(), r#""ls""#);
    }

    #[test]
    fn test_tool_kind_deserialize_all() {
        let kinds = [
            ("read-file", ToolKind::ReadFile),
            ("edit-file", ToolKind::EditFile),
            ("shell", ToolKind::Shell),
            ("web-fetch", ToolKind::WebFetch),
            ("glob", ToolKind::Glob),
            ("grep", ToolKind::Grep),
            ("ls", ToolKind::Ls),
        ];
        for (json_str, expected) in kinds {
            let kind: ToolKind = serde_json::from_str(&format!(r#""{}""#, json_str)).unwrap();
            assert_eq!(kind, expected, "Failed for {}", json_str);
        }
    }

    #[test]
    fn test_tool_call_status_serialize_all() {
        assert_eq!(
            serde_json::to_string(&ToolCallStatus::Pending).unwrap(),
            r#""pending""#
        );
        assert_eq!(
            serde_json::to_string(&ToolCallStatus::InProgress).unwrap(),
            r#""in-progress""#
        );
        assert_eq!(
            serde_json::to_string(&ToolCallStatus::Success).unwrap(),
            r#""success""#
        );
        assert_eq!(
            serde_json::to_string(&ToolCallStatus::Error).unwrap(),
            r#""error""#
        );
        assert_eq!(
            serde_json::to_string(&ToolCallStatus::Cancelled).unwrap(),
            r#""cancelled""#
        );
    }

    #[test]
    fn test_tool_call_status_deserialize_all() {
        let statuses = [
            ("pending", ToolCallStatus::Pending),
            ("in-progress", ToolCallStatus::InProgress),
            ("success", ToolCallStatus::Success),
            ("error", ToolCallStatus::Error),
            ("cancelled", ToolCallStatus::Cancelled),
        ];
        for (json_str, expected) in statuses {
            let status: ToolCallStatus =
                serde_json::from_str(&format!(r#""{}""#, json_str)).unwrap();
            assert_eq!(status, expected, "Failed for {}", json_str);
        }
    }

    #[test]
    fn test_tool_call_update_serialization() {
        let update = ToolCallUpdate {
            tool_call_id: "tc-1".to_string(),
            status: ToolCallStatus::InProgress,
            content: Some(serde_json::json!({"output": "running..."})),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"toolCallId\":\"tc-1\""));
        assert!(json.contains("\"status\":\"in-progress\""));
        assert!(json.contains("\"content\""));
    }

    #[test]
    fn test_tool_call_update_no_content() {
        let update = ToolCallUpdate {
            tool_call_id: "tc-2".to_string(),
            status: ToolCallStatus::Success,
            content: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(!json.contains("\"content\""));
    }

    #[test]
    fn test_tool_call_update_deserialization() {
        let json = r#"{"toolCallId":"tc-3","status":"error"}"#;
        let update: ToolCallUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.tool_call_id, "tc-3");
        assert_eq!(update.status, ToolCallStatus::Error);
        assert!(update.content.is_none());
    }

    #[test]
    fn test_tool_call_location_serialization() {
        let loc = ToolCallLocation {
            tool_call_id: "tc-4".to_string(),
            path: Some("src/main.rs".to_string()),
            line: Some(42),
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("\"path\":\"src/main.rs\""));
        assert!(json.contains("\"line\":42"));
    }

    #[test]
    fn test_tool_call_location_skip_none() {
        let loc = ToolCallLocation {
            tool_call_id: "tc-5".to_string(),
            path: None,
            line: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(!json.contains("\"path\""));
        assert!(!json.contains("\"line\""));
    }

    #[test]
    fn test_tool_call_location_partial() {
        let loc = ToolCallLocation {
            tool_call_id: "tc-6".to_string(),
            path: Some("lib.rs".to_string()),
            line: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("\"path\":\"lib.rs\""));
        assert!(!json.contains("\"line\""));
    }

    #[test]
    fn test_tool_call_location_deserialization() {
        let json = r#"{"toolCallId":"tc-7","path":"foo.rs","line":10}"#;
        let loc: ToolCallLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.tool_call_id, "tc-7");
        assert_eq!(loc.path, Some("foo.rs".to_string()));
        assert_eq!(loc.line, Some(10));
    }
}
