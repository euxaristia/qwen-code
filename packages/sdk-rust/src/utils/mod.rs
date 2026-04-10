//! Utility functions and types for the ACP SDK.

use std::time::Duration;

/// Timeout configuration for various operations.
#[derive(Debug, Clone, Copy)]
pub struct Timeouts {
    /// Timeout for a full conversation turn (prompt → response).
    pub turn: Duration,
    /// Timeout for a single message response.
    pub message: Duration,
    /// Timeout for event consumption (handling agent requests).
    pub event: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            turn: Duration::from_secs(30 * 60), // 30 minutes
            message: Duration::from_secs(180),  // 3 minutes
            event: Duration::from_secs(60),     // 60 seconds
        }
    }
}

/// Generate a unique request ID.
pub fn generate_id() -> String {
    format!("req-{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeouts() {
        let timeouts = Timeouts::default();
        assert_eq!(timeouts.turn, Duration::from_secs(30 * 60));
        assert_eq!(timeouts.message, Duration::from_secs(180));
        assert_eq!(timeouts.event, Duration::from_secs(60));
    }

    #[test]
    fn test_generate_id_format() {
        let id = generate_id();
        assert!(id.starts_with("req-"));
        assert!(id.len() > 10); // UUID is 36 chars, so total > 40
    }

    #[test]
    fn test_generate_id_uniqueness() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
    }
}
