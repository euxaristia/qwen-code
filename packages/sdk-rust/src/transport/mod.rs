//! Transport layer for ACP communication.

use async_trait::async_trait;
use std::io;
use thiserror::Error;

pub mod process;

/// Transport errors.
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Transport not started")]
    NotStarted,

    #[error("Transport closed")]
    Closed,

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Protocol error: {0}")]
    Protocol(String),
}

/// Callback type for stream processing — returns `true` to stop.
pub type StreamCallback = Box<dyn FnMut(&str) -> bool + Send>;

/// Transport trait for ACP communication.
///
/// Implementations provide bidirectional communication with an ACP agent,
/// typically via stdin/stdout of a subprocess.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Start the transport.
    async fn start(&mut self) -> Result<(), TransportError>;

    /// Close the transport and release resources.
    async fn close(&mut self) -> Result<(), TransportError>;

    /// Check if the transport is currently available.
    fn is_available(&self) -> bool;

    /// Send a message and wait for a single-line response.
    async fn request(&mut self, message: &str) -> Result<String, TransportError>;

    /// Send a message and process multiple response lines via a callback.
    /// The callback returns `true` to stop reading.
    async fn request_stream(
        &mut self,
        message: &str,
        callback: StreamCallback,
    ) -> Result<(), TransportError>;

    /// Send a message without waiting for a response.
    async fn send(&mut self, message: &str) -> Result<(), TransportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_error_io_display() {
        let err = TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken pipe",
        ));
        let display = format!("{}", err);
        assert!(display.contains("I/O error"));
        assert!(display.contains("broken pipe"));
    }

    #[test]
    fn test_transport_error_not_started_display() {
        let err = TransportError::NotStarted;
        assert_eq!(format!("{}", err), "Transport not started");
    }

    #[test]
    fn test_transport_error_closed_display() {
        let err = TransportError::Closed;
        assert_eq!(format!("{}", err), "Transport closed");
    }

    #[test]
    fn test_transport_error_timeout_display() {
        let err = TransportError::Timeout;
        assert_eq!(format!("{}", err), "Timeout waiting for response");
    }

    #[test]
    fn test_transport_error_protocol_display() {
        let err = TransportError::Protocol("bad format".to_string());
        assert_eq!(format!("{}", err), "Protocol error: bad format");
    }

    #[test]
    fn test_transport_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(TransportError::Timeout);
        assert_eq!(format!("{}", err), "Timeout waiting for response");
    }

    #[test]
    fn test_transport_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "fail");
        let err: TransportError = TransportError::from(io_err);
        match err {
            TransportError::Io(_) => {} // expected
            _ => panic!("Expected Io variant"),
        }
    }
}
