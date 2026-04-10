//! Process-based transport that communicates with an ACP agent via subprocess stdio.

use crate::transport::{StreamCallback, Transport, TransportError};
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, trace, warn};

/// Default timeout for a full conversation turn.
pub const DEFAULT_TURN_TIMEOUT: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Default timeout for a single message response.
pub const DEFAULT_MESSAGE_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes

/// Options for configuring a [`ProcessTransport`].
#[derive(Debug, Clone)]
pub struct ProcessTransportOptions {
    /// Command and arguments to launch the agent process.
    pub command: Vec<String>,
    /// Working directory for the subprocess.
    pub cwd: Option<String>,
    /// Turn timeout.
    pub turn_timeout: Option<Duration>,
    /// Per-message timeout.
    pub message_timeout: Option<Duration>,
}

impl Default for ProcessTransportOptions {
    fn default() -> Self {
        Self {
            command: vec!["qwen".to_string(), "--acp".to_string(), "-y".to_string()],
            cwd: None,
            turn_timeout: None,
            message_timeout: None,
        }
    }
}

/// Process-based transport implementation.
pub struct ProcessTransport {
    opts: ProcessTransportOptions,
    child: Option<Child>,
    stdin: Option<tokio::process::ChildStdin>,
    stdout: Option<BufReader<tokio::process::ChildStdout>>,
    running: Arc<AtomicBool>,
}

impl ProcessTransport {
    /// Create a new ProcessTransport with the given options.
    pub fn new(opts: ProcessTransportOptions) -> Self {
        Self {
            opts,
            child: None,
            stdin: None,
            stdout: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Spawn the error stream reader task.
    fn spawn_error_reader(child: &mut Child) {
        let stderr = child.stderr.take();
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trimmed = line.trim_end();
                            if !trimmed.is_empty() {
                                warn!(stderr = trimmed, "subprocess stderr");
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "error reading stderr");
                            break;
                        }
                    }
                }
            });
        }
    }
}

#[async_trait]
impl Transport for ProcessTransport {
    async fn start(&mut self) -> Result<(), TransportError> {
        if self.child.is_some() {
            return Ok(());
        }

        let mut cmd = Command::new(&self.opts.command[0]);
        cmd.args(&self.opts.command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(cwd) = &self.opts.cwd {
            cmd.current_dir(Path::new(cwd));
        }

        debug!(command = ?self.opts.command, cwd = ?self.opts.cwd, "starting process transport");

        let mut child = cmd.spawn().map_err(TransportError::Io)?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TransportError::Protocol("Failed to capture subprocess stdin".into()))?;

        let stdout = child.stdout.take().ok_or_else(|| {
            TransportError::Protocol("Failed to capture subprocess stdout".into())
        })?;

        Self::spawn_error_reader(&mut child);

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(BufReader::new(stdout));
        self.running.store(true, Ordering::SeqCst);

        debug!("process transport started");
        Ok(())
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        if let Some(mut child) = self.child.take() {
            debug!("closing process transport");
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.stdin = None;
        self.stdout = None;
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.child
            .as_ref()
            .map(|c| c.id().is_some())
            .unwrap_or(false)
            && self.running.load(Ordering::SeqCst)
    }

    async fn request(&mut self, message: &str) -> Result<String, TransportError> {
        self.send(message).await?;
        self.read_line().await
    }

    async fn request_stream(
        &mut self,
        message: &str,
        mut callback: StreamCallback,
    ) -> Result<(), TransportError> {
        self.send(message).await?;

        let timeout = self.opts.message_timeout.unwrap_or(DEFAULT_MESSAGE_TIMEOUT);

        loop {
            let line = tokio::time::timeout(timeout, self.read_line_inner()).await;
            match line {
                Ok(Ok(line)) => {
                    trace!(line = %line, "stream read line");
                    if callback(&line) {
                        break;
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    warn!(?timeout, "timeout waiting for stream line");
                    return Err(TransportError::Timeout);
                }
            }
        }

        Ok(())
    }

    async fn send(&mut self, message: &str) -> Result<(), TransportError> {
        let stdin = self.stdin.as_mut().ok_or(TransportError::NotStarted)?;

        trace!(message, "sending message to subprocess");
        stdin
            .write_all(message.as_bytes())
            .await
            .map_err(TransportError::Io)?;
        stdin.write_all(b"\n").await.map_err(TransportError::Io)?;
        stdin.flush().await.map_err(TransportError::Io)?;
        Ok(())
    }
}

impl ProcessTransport {
    /// Read a single line from stdout with message timeout.
    async fn read_line(&mut self) -> Result<String, TransportError> {
        let timeout = self.opts.message_timeout.unwrap_or(DEFAULT_MESSAGE_TIMEOUT);
        tokio::time::timeout(timeout, self.read_line_inner())
            .await
            .map_err(|_| TransportError::Timeout)?
    }

    /// Read a single line from stdout without sending first (for streaming).
    pub(crate) async fn request_inner(&mut self) -> Result<String, TransportError> {
        let timeout = self.opts.message_timeout.unwrap_or(DEFAULT_MESSAGE_TIMEOUT);
        tokio::time::timeout(timeout, self.read_line_inner())
            .await
            .map_err(|_| TransportError::Timeout)?
    }

    /// Read a single line from stdout without timeout.
    async fn read_line_inner(&mut self) -> Result<String, TransportError> {
        let stdout = self.stdout.as_mut().ok_or(TransportError::NotStarted)?;

        let mut line = String::new();
        let n = stdout
            .read_line(&mut line)
            .await
            .map_err(TransportError::Io)?;

        if n == 0 {
            return Err(TransportError::Closed);
        }

        Ok(line.trim_end().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = ProcessTransportOptions::default();
        assert_eq!(opts.command, vec!["qwen", "--acp", "-y"]);
        assert!(opts.cwd.is_none());
    }

    #[test]
    fn test_custom_options() {
        let opts = ProcessTransportOptions {
            command: vec!["custom".to_string(), "--flag".to_string()],
            cwd: Some("/tmp".to_string()),
            ..Default::default()
        };
        assert_eq!(opts.command, vec!["custom", "--flag"]);
        assert_eq!(opts.cwd.as_deref(), Some("/tmp"));
    }
}
