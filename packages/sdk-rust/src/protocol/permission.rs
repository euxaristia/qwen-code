//! Permission domain types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionOptionKind {
    Allow,
    Deny,
    AllowAlways,
    DenyAlways,
    Interactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOption {
    pub kind: PermissionOptionKind,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPermissionOutcome {
    pub option: PermissionOption,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_option_kind_serialize() {
        assert_eq!(
            serde_json::to_string(&PermissionOptionKind::Allow).unwrap(),
            r#""Allow""#
        );
        assert_eq!(
            serde_json::to_string(&PermissionOptionKind::Deny).unwrap(),
            r#""Deny""#
        );
        assert_eq!(
            serde_json::to_string(&PermissionOptionKind::AllowAlways).unwrap(),
            r#""AllowAlways""#
        );
        assert_eq!(
            serde_json::to_string(&PermissionOptionKind::DenyAlways).unwrap(),
            r#""DenyAlways""#
        );
        assert_eq!(
            serde_json::to_string(&PermissionOptionKind::Interactive).unwrap(),
            r#""Interactive""#
        );
    }

    #[test]
    fn test_permission_option_kind_deserialize() {
        let kind: PermissionOptionKind = serde_json::from_str(r#""Allow""#).unwrap();
        assert_eq!(kind, PermissionOptionKind::Allow);

        let kind: PermissionOptionKind = serde_json::from_str(r#""Deny""#).unwrap();
        assert_eq!(kind, PermissionOptionKind::Deny);

        let kind: PermissionOptionKind = serde_json::from_str(r#""AllowAlways""#).unwrap();
        assert_eq!(kind, PermissionOptionKind::AllowAlways);
    }

    #[test]
    fn test_permission_option_serialization() {
        let option = PermissionOption {
            kind: PermissionOptionKind::AllowAlways,
            label: "Allow for this session".to_string(),
        };
        let json = serde_json::to_string(&option).unwrap();
        assert!(json.contains("\"kind\":\"AllowAlways\""));
        assert!(json.contains("\"label\":\"Allow for this session\""));
    }

    #[test]
    fn test_permission_option_deserialization() {
        let json = r#"{"kind":"Deny","label":"Deny"}"#;
        let option: PermissionOption = serde_json::from_str(json).unwrap();
        assert_eq!(option.kind, PermissionOptionKind::Deny);
        assert_eq!(option.label, "Deny");
    }

    #[test]
    fn test_request_permission_outcome() {
        let outcome = RequestPermissionOutcome {
            option: PermissionOption {
                kind: PermissionOptionKind::Allow,
                label: "Yes".to_string(),
            },
        };
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("\"option\""));
        assert!(json.contains("\"kind\":\"Allow\""));
    }
}
