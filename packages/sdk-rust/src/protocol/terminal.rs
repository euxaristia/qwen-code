//! Terminal domain types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalResult {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputResult {
    pub output: String,
    #[serde(rename = "isRunning")]
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_variable_serialization() {
        let env = EnvVariable {
            key: "PATH".to_string(),
            value: "/usr/bin".to_string(),
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"key\":\"PATH\""));
        assert!(json.contains("\"value\":\"/usr/bin\""));
    }

    #[test]
    fn test_env_variable_deserialization() {
        let json = r#"{"key":"HOME","value":"/home/user"}"#;
        let env: EnvVariable = serde_json::from_str(json).unwrap();
        assert_eq!(env.key, "HOME");
        assert_eq!(env.value, "/home/user");
    }

    #[test]
    fn test_create_terminal_result_serialization() {
        let result = CreateTerminalResult {
            terminal_id: "term-1".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"terminalId\":\"term-1\""));
    }

    #[test]
    fn test_create_terminal_result_deserialization() {
        let json = r#"{"terminalId":"term-42"}"#;
        let result: CreateTerminalResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.terminal_id, "term-42");
    }

    #[test]
    fn test_terminal_output_result_serialization() {
        let result = TerminalOutputResult {
            output: "line1\nline2\n".to_string(),
            is_running: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"isRunning\":true"));
        assert!(json.contains("\"output\""));
    }

    #[test]
    fn test_terminal_output_result_deserialization() {
        let json = r#"{"output":"hello\n","isRunning":false}"#;
        let result: TerminalOutputResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.output, "hello\n");
        assert!(!result.is_running);
    }

    #[test]
    fn test_terminal_output_not_running() {
        let result = TerminalOutputResult {
            output: "".to_string(),
            is_running: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"isRunning\":false"));
    }
}
