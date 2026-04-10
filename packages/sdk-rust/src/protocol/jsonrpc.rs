//! JSON-RPC 2.0 base types used by the ACP protocol.

use serde::{Deserialize, Serialize};

/// JSON-RPC version used throughout the protocol.
pub const JSONRPC_VERSION: &str = "2.0";

/// Base message trait — all JSON-RPC messages share `jsonrpc` and `id` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: None,
        }
    }

    pub fn with_id(id: String) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id),
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

/// A JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request<P> {
    #[serde(flatten)]
    pub base: Message,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<P>,
}

impl<P> Request<P> {
    pub fn new(method: impl Into<String>, params: P) -> Self {
        Self {
            base: Message::new(),
            method: method.into(),
            params: Some(params),
        }
    }
}

/// A JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<R> {
    #[serde(flatten)]
    pub base: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<R>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

impl<R> Response<R> {
    pub fn result(result: R) -> Self {
        Self {
            base: Message::new(),
            result: Some(result),
            error: None,
        }
    }

    pub fn error(error: Error) -> Self {
        Self {
            base: Message::new(),
            result: None,
            error: Some(error),
        }
    }
}

/// A JSON-RPC notification (no `id`, no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification<N> {
    #[serde(flatten)]
    pub base: Message,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<N>,
}

impl<N> Notification<N> {
    pub fn new(method: impl Into<String>, params: N) -> Self {
        Self {
            base: Message::new(),
            method: method.into(),
            params: Some(params),
        }
    }
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    /// Standard error codes used in ACP.
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;

    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(code: i64, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC error ({}): {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::with_id("abc123".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":\"abc123\""));
    }

    #[test]
    fn test_message_default() {
        let msg = Message::default();
        assert_eq!(msg.jsonrpc, JSONRPC_VERSION);
        assert!(msg.id.is_none());
    }

    #[test]
    fn test_message_new() {
        let msg = Message::new();
        assert_eq!(msg.jsonrpc, JSONRPC_VERSION);
        assert!(msg.id.is_none());
    }

    #[test]
    fn test_message_with_id() {
        let msg = Message::with_id("req-42".to_string());
        assert_eq!(msg.id, Some("req-42".to_string()));
    }

    #[test]
    fn test_request_roundtrip() {
        #[derive(Serialize, Deserialize, Debug)]
        struct TestParams {
            foo: String,
        }

        let req: Request<TestParams> = Request::new(
            "test.method",
            TestParams {
                foo: "bar".to_string(),
            },
        );
        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request<TestParams> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "test.method");
        assert_eq!(parsed.params.unwrap().foo, "bar");
    }

    #[test]
    fn test_request_has_jsonrpc_version() {
        #[derive(Serialize, Deserialize)]
        struct EmptyParams {}

        let req: Request<EmptyParams> = Request::new("test", EmptyParams {});
        assert_eq!(req.base.jsonrpc, JSONRPC_VERSION);
    }

    #[test]
    fn test_response_result_serialization() {
        let resp = Response::result("success".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"result\":\"success\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_response_error_serialization() {
        let resp: Response<String> = Response::error(Error::new(-1, "failed"));
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("\"code\":-1"));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn test_response_deserialization_with_result() {
        let json = r#"{"jsonrpc":"2.0","id":"1","result":{"value":42}}"#;
        #[derive(Deserialize, Debug, PartialEq)]
        struct ValueResult {
            value: i64,
        }
        let resp: Response<ValueResult> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.result.unwrap().value, 42);
        assert!(resp.error.is_none());
        assert_eq!(resp.base.id, Some("1".to_string()));
    }

    #[test]
    fn test_response_deserialization_with_error() {
        let json =
            r#"{"jsonrpc":"2.0","id":"1","error":{"code":-32600,"message":"Invalid request"}}"#;
        let resp: Response<String> = serde_json::from_str(json).unwrap();
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32600);
    }

    #[test]
    fn test_notification_serialization() {
        let notif = Notification::new("session/update", "test".to_string());
        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("\"method\":\"session/update\""));
        assert!(json.contains("\"params\":\"test\""));
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_notification_deserialization() {
        let json = r#"{"jsonrpc":"2.0","method":"notify","params":{"key":"val"}}"#;
        #[derive(Deserialize, Debug)]
        struct TestParams {
            key: String,
        }
        let notif: Notification<TestParams> = serde_json::from_str(json).unwrap();
        assert_eq!(notif.method, "notify");
        assert_eq!(notif.params.unwrap().key, "val");
        assert!(notif.base.id.is_none());
    }

    #[test]
    fn test_error_new() {
        let err = Error::new(Error::PARSE_ERROR, "Parse error");
        assert_eq!(err.code, -32700);
        assert_eq!(err.message, "Parse error");
        assert!(err.data.is_none());
    }

    #[test]
    fn test_error_with_data() {
        let data = serde_json::json!({"detail": "something"});
        let err = Error::with_data(-32603, "Internal", data.clone());
        assert_eq!(err.code, -32603);
        assert_eq!(err.message, "Internal");
        assert_eq!(err.data, Some(data));
    }

    #[test]
    fn test_error_all_codes() {
        assert_eq!(Error::PARSE_ERROR, -32700);
        assert_eq!(Error::INVALID_REQUEST, -32600);
        assert_eq!(Error::METHOD_NOT_FOUND, -32601);
        assert_eq!(Error::INVALID_PARAMS, -32602);
        assert_eq!(Error::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_error_deserialization() {
        let json = r#"{"code":-32601,"message":"Method not found","data":{"method":"unknown"}}"#;
        let err: Error = serde_json::from_str(json).unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
        assert!(err.data.is_some());
        assert_eq!(err.data.unwrap()["method"], "unknown");
    }

    #[test]
    fn test_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(Error::new(-1, "fail"));
        assert_eq!(format!("{}", err), "JSON-RPC error (-1): fail");
    }

    #[test]
    fn test_request_optional_params() {
        #[derive(Serialize, Deserialize, Debug)]
        struct Empty {}
        let req = Request {
            base: Message::new(),
            method: "test".to_string(),
            params: None::<Empty>,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("\"params\""));
    }
}
