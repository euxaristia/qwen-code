//! Plan domain types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PlanEntryPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PlanEntryStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEntry {
    pub content: String,
    pub priority: PlanEntryPriority,
    pub status: PlanEntryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub entries: Vec<PlanEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_entry_priority_serialize() {
        assert_eq!(
            serde_json::to_string(&PlanEntryPriority::High).unwrap(),
            r#""high""#
        );
        assert_eq!(
            serde_json::to_string(&PlanEntryPriority::Medium).unwrap(),
            r#""medium""#
        );
        assert_eq!(
            serde_json::to_string(&PlanEntryPriority::Low).unwrap(),
            r#""low""#
        );
    }

    #[test]
    fn test_plan_entry_priority_deserialize() {
        let high: PlanEntryPriority = serde_json::from_str(r#""high""#).unwrap();
        assert_eq!(high, PlanEntryPriority::High);

        let med: PlanEntryPriority = serde_json::from_str(r#""medium""#).unwrap();
        assert_eq!(med, PlanEntryPriority::Medium);
    }

    #[test]
    fn test_plan_entry_status_serialize() {
        assert_eq!(
            serde_json::to_string(&PlanEntryStatus::Pending).unwrap(),
            r#""pending""#
        );
        assert_eq!(
            serde_json::to_string(&PlanEntryStatus::InProgress).unwrap(),
            r#""in-progress""#
        );
        assert_eq!(
            serde_json::to_string(&PlanEntryStatus::Completed).unwrap(),
            r#""completed""#
        );
        assert_eq!(
            serde_json::to_string(&PlanEntryStatus::Cancelled).unwrap(),
            r#""cancelled""#
        );
    }

    #[test]
    fn test_plan_entry_status_deserialize() {
        let completed: PlanEntryStatus = serde_json::from_str(r#""completed""#).unwrap();
        assert_eq!(completed, PlanEntryStatus::Completed);

        let in_progress: PlanEntryStatus = serde_json::from_str(r#""in-progress""#).unwrap();
        assert_eq!(in_progress, PlanEntryStatus::InProgress);
    }

    #[test]
    fn test_plan_entry_serialization() {
        let entry = PlanEntry {
            content: "Implement feature X".to_string(),
            priority: PlanEntryPriority::High,
            status: PlanEntryStatus::InProgress,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"content\":\"Implement feature X\""));
        assert!(json.contains("\"priority\":\"high\""));
        assert!(json.contains("\"status\":\"in-progress\""));
    }

    #[test]
    fn test_plan_entry_deserialization() {
        let json = r#"{"content":"Write tests","priority":"low","status":"pending"}"#;
        let entry: PlanEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.content, "Write tests");
        assert_eq!(entry.priority, PlanEntryPriority::Low);
        assert_eq!(entry.status, PlanEntryStatus::Pending);
    }

    #[test]
    fn test_plan_serialization() {
        let plan = Plan {
            entries: vec![
                PlanEntry {
                    content: "Step 1".to_string(),
                    priority: PlanEntryPriority::High,
                    status: PlanEntryStatus::Completed,
                },
                PlanEntry {
                    content: "Step 2".to_string(),
                    priority: PlanEntryPriority::Medium,
                    status: PlanEntryStatus::Pending,
                },
            ],
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"entries\""));
        assert!(json.contains("\"Step 1\""));
        assert!(json.contains("\"Step 2\""));
    }

    #[test]
    fn test_plan_deserialization() {
        let json = r#"{"entries":[{"content":"task1","priority":"high","status":"done"}]}"#;
        // Note: "done" is not a valid PlanEntryStatus, so this should fail
        let result: Result<Plan, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_plan_empty_entries() {
        let plan = Plan { entries: vec![] };
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, r#"{"entries":[]}"#);
    }

    #[test]
    fn test_plan_roundtrip() {
        let plan = Plan {
            entries: vec![PlanEntry {
                content: "Roundtrip test".to_string(),
                priority: PlanEntryPriority::Low,
                status: PlanEntryStatus::Pending,
            }],
        };
        let json = serde_json::to_string(&plan).unwrap();
        let parsed: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].content, "Roundtrip test");
        assert_eq!(parsed.entries[0].priority, PlanEntryPriority::Low);
    }
}
