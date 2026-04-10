//! Session domain types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SessionMode {
    Normal,
    Plan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModeState {
    pub mode: SessionMode,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StopReason {
    EndTurn,
    Cancelled,
    MaxTokens,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    #[serde(rename = "stopReason")]
    pub stop_reason: StopReason,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&SessionMode::Normal).unwrap(),
            r#""normal""#
        );
        assert_eq!(
            serde_json::to_string(&SessionMode::Plan).unwrap(),
            r#""plan""#
        );
    }

    #[test]
    fn test_session_mode_deserialize() {
        let mode: SessionMode = serde_json::from_str(r#""normal""#).unwrap();
        assert_eq!(mode, SessionMode::Normal);

        let mode: SessionMode = serde_json::from_str(r#""plan""#).unwrap();
        assert_eq!(mode, SessionMode::Plan);
    }

    #[test]
    fn test_session_mode_state_serialization() {
        let state = SessionModeState {
            mode: SessionMode::Plan,
            is_default: false,
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"mode\":\"plan\""));
        assert!(json.contains("\"isDefault\":false"));
    }

    #[test]
    fn test_session_mode_state_deserialization() {
        let json = r#"{"mode":"normal","isDefault":true}"#;
        let state: SessionModeState = serde_json::from_str(json).unwrap();
        assert_eq!(state.mode, SessionMode::Normal);
        assert!(state.is_default);
    }

    #[test]
    fn test_stop_reason_serialize_all() {
        assert_eq!(
            serde_json::to_string(&StopReason::EndTurn).unwrap(),
            r#""end-turn""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::Cancelled).unwrap(),
            r#""cancelled""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::MaxTokens).unwrap(),
            r#""max-tokens""#
        );
        assert_eq!(
            serde_json::to_string(&StopReason::Error).unwrap(),
            r#""error""#
        );
    }

    #[test]
    fn test_stop_reason_deserialize_all() {
        let reasons = [
            ("end-turn", StopReason::EndTurn),
            ("cancelled", StopReason::Cancelled),
            ("max-tokens", StopReason::MaxTokens),
            ("error", StopReason::Error),
        ];
        for (json_str, expected) in reasons {
            let reason: StopReason = serde_json::from_str(&format!(r#""{}""#, json_str)).unwrap();
            assert_eq!(reason, expected, "Failed for {}", json_str);
        }
    }

    #[test]
    fn test_prompt_result_serialization() {
        let result = PromptResult {
            stop_reason: StopReason::EndTurn,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"stopReason\":\"end-turn\""));
    }

    #[test]
    fn test_prompt_result_deserialization() {
        let json = r#"{"stopReason":"cancelled"}"#;
        let result: PromptResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.stop_reason, StopReason::Cancelled);
    }

    #[test]
    fn test_prompt_result_max_tokens() {
        let json = r#"{"stopReason":"max-tokens"}"#;
        let result: PromptResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.stop_reason, StopReason::MaxTokens);
    }

    #[test]
    fn test_session_mode_equality() {
        assert_eq!(SessionMode::Normal, SessionMode::Normal);
        assert_ne!(SessionMode::Normal, SessionMode::Plan);
    }

    #[test]
    fn test_stop_reason_equality() {
        assert_eq!(StopReason::EndTurn, StopReason::EndTurn);
        assert_ne!(StopReason::EndTurn, StopReason::Cancelled);
    }

    #[test]
    fn test_invalid_session_mode_fails() {
        let result: Result<SessionMode, _> = serde_json::from_str(r#""invalid""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_stop_reason_fails() {
        let result: Result<StopReason, _> = serde_json::from_str(r#""timeout""#);
        assert!(result.is_err());
    }
}
