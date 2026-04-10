//! # ACP SDK — Rust implementation of the Agent Client Protocol
//!
//! This crate provides a Rust SDK for communicating with AI agents via the
//! Agent Client Protocol (ACP), a JSON-RPC 2.0-based protocol implemented by
//! the Qwen Code CLI.
//!
//! ## Quick Start
//!
//! ```no_run
//! use acp_sdk::AcpClient;
//! use acp_sdk::transport::process::ProcessTransportOptions;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let opts = ProcessTransportOptions::default();
//! let mut client = AcpClient::with_options(opts).await?;
//! client.new_session().await?;
//! let results = client.send_prompt_text(&["Hello!".to_string().into()]).await?;
//! client.close().await?;
//! # Ok(())
//! # }
//! ```

pub mod protocol;
pub mod session;
pub mod transport;
pub mod utils;

mod client;

pub use client::{AcpClient, ClientError, NewSessionParams};
pub use session::{LoadSessionParams, Session, SessionError};
