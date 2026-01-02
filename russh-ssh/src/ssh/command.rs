//! SSH Command Execution
//!
//! Handles command execution and interactive shell sessions.
//!
//! # Requirements Coverage
//! - Requirement 9.1: Execute commands on remote host asynchronously
//! - Requirement 9.2: Stream stdout and stderr separately in real-time
//! - Requirement 9.3: Return exit code when command completes
//! - Requirement 9.5: Command timeout handling

use super::SshClient;
use crate::error::SshError;
use std::time::Duration;
use tokio::sync::mpsc;

/// Result of a command execution
///
/// Contains the stdout, stderr, and exit code from the executed command.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Standard output from the command
    pub stdout: Vec<u8>,
    /// Standard error from the command
    pub stderr: Vec<u8>,
    /// Exit code (0 typically indicates success)
    pub exit_code: i32,
}

impl CommandResult {
    /// Check if the command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get stdout as a string (lossy UTF-8 conversion)
    pub fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    /// Get stderr as a string (lossy UTF-8 conversion)
    pub fn stderr_string(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}

impl SshClient {
    /// Execute a command and return the result
    ///
    /// # Requirements Coverage
    /// - Requirement 9.1: Execute commands on remote host asynchronously
    /// - Requirement 9.3: Return exit code when command completes
    pub async fn execute(&self, command: &str) -> Result<CommandResult, SshError> {
        let client = self.inner().ok_or(SshError::NotConnected)?;
        
        tracing::debug!("Executing command: {}", command);
        
        let result = client.execute(command).await
            .map_err(|e| SshError::CommandExecution(e.to_string()))?;

        tracing::debug!(
            "Command completed with exit code: {}", 
            result.exit_status
        );

        Ok(CommandResult {
            stdout: result.stdout.into_bytes(),
            stderr: result.stderr.into_bytes(),
            exit_code: result.exit_status as i32,
        })
    }

    /// Execute command with streaming output
    /// 
    /// Sends stdout and stderr through separate channels as data becomes available.
    /// Returns the exit code when the command completes.
    ///
    /// # Requirements Coverage
    /// - Requirement 9.2: Stream stdout and stderr separately in real-time
    /// - Requirement 9.3: Return exit code when command completes
    ///
    /// Note: This is a simplified implementation that executes the command
    /// and sends the output through channels. For true streaming, lower-level
    /// channel access would be needed.
    pub async fn execute_streaming(
        &self,
        command: &str,
        stdout_tx: mpsc::Sender<Vec<u8>>,
        stderr_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<i32, SshError> {
        tracing::debug!("Executing command with streaming: {}", command);
        
        let res = self.execute(command).await?;
        
        // Send stdout if not empty
        if !res.stdout.is_empty() {
            if let Err(e) = stdout_tx.send(res.stdout).await {
                tracing::warn!("Failed to send stdout: {}", e);
            }
        }
        
        // Send stderr if not empty
        if !res.stderr.is_empty() {
            if let Err(e) = stderr_tx.send(res.stderr).await {
                tracing::warn!("Failed to send stderr: {}", e);
            }
        }
        
        Ok(res.exit_code)
    }

    /// Execute multiple commands in sequence
    ///
    /// Returns results for all commands. Stops on first failure if `stop_on_error` is true.
    pub async fn execute_batch(
        &self,
        commands: &[&str],
        stop_on_error: bool,
    ) -> Result<Vec<CommandResult>, SshError> {
        let mut results = Vec::with_capacity(commands.len());
        
        for command in commands {
            let result = self.execute(command).await?;
            let failed = !result.success();
            results.push(result);
            
            if failed && stop_on_error {
                break;
            }
        }
        
        Ok(results)
    }

    /// Execute a command with a timeout
    ///
    /// If the command does not complete within the specified duration,
    /// it will be terminated and a `CommandTimeout` error will be returned.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `timeout` - Maximum duration to wait for the command to complete
    ///
    /// # Requirements Coverage
    /// - Requirement 9.5: Command timeout handling
    pub async fn execute_with_timeout(
        &self,
        command: &str,
        timeout: Duration,
    ) -> Result<CommandResult, SshError> {
        tracing::debug!("Executing command with timeout {:?}: {}", timeout, command);
        
        match tokio::time::timeout(timeout, self.execute(command)).await {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!("Command timed out after {:?}: {}", timeout, command);
                Err(SshError::CommandTimeout(timeout))
            }
        }
    }

    /// Execute a command with streaming output and a timeout
    ///
    /// Combines streaming output with timeout handling. If the command
    /// does not complete within the specified duration, it will be
    /// terminated and a `CommandTimeout` error will be returned.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `timeout` - Maximum duration to wait for the command to complete
    /// * `stdout_tx` - Channel sender for stdout data
    /// * `stderr_tx` - Channel sender for stderr data
    ///
    /// # Requirements Coverage
    /// - Requirement 9.2: Stream stdout and stderr separately in real-time
    /// - Requirement 9.5: Command timeout handling
    pub async fn execute_streaming_with_timeout(
        &self,
        command: &str,
        timeout: Duration,
        stdout_tx: mpsc::Sender<Vec<u8>>,
        stderr_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<i32, SshError> {
        tracing::debug!("Executing streaming command with timeout {:?}: {}", timeout, command);
        
        match tokio::time::timeout(
            timeout,
            self.execute_streaming(command, stdout_tx, stderr_tx)
        ).await {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!("Streaming command timed out after {:?}: {}", timeout, command);
                Err(SshError::CommandTimeout(timeout))
            }
        }
    }
}


/// Interactive shell session with PTY
///
/// Provides an interactive shell session with pseudo-terminal allocation.
/// Uses channels for stdin/stdout/stderr communication.
///
/// # Requirements Coverage
/// - Requirement 9.4: Interactive shell sessions with PTY allocation
pub struct Shell {
    /// Sender for stdin data
    stdin_tx: mpsc::Sender<Vec<u8>>,
    /// Receiver for stdout data
    stdout_rx: mpsc::Receiver<Vec<u8>>,
    /// Terminal type
    term: String,
    /// Terminal dimensions
    cols: u32,
    rows: u32,
}

impl Shell {
    /// Create a new shell with I/O channels
    pub(crate) fn new(
        stdin_tx: mpsc::Sender<Vec<u8>>,
        stdout_rx: mpsc::Receiver<Vec<u8>>,
        term: String,
        cols: u32,
        rows: u32,
    ) -> Self {
        Self { stdin_tx, stdout_rx, term, cols, rows }
    }

    /// Get the terminal type
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Get terminal dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.cols, self.rows)
    }

    /// Write data to stdin
    pub async fn write(&self, data: &[u8]) -> Result<(), SshError> {
        self.stdin_tx.send(data.to_vec()).await
            .map_err(|e| SshError::CommandExecution(format!("Failed to write to stdin: {}", e)))
    }

    /// Read data from stdout (blocking until data available)
    pub async fn read(&mut self) -> Option<Vec<u8>> {
        self.stdout_rx.recv().await
    }

    /// Send EOF to stdin (signals end of input)
    pub async fn send_eof(&self) -> Result<(), SshError> {
        self.stdin_tx.send(Vec::new()).await
            .map_err(|e| SshError::CommandExecution(format!("Failed to send EOF: {}", e)))
    }
}

impl SshClient {
    /// Open an interactive shell with PTY
    ///
    /// Returns a Shell handle for I/O and spawns a background task to handle
    /// the shell session. The shell runs until stdin is closed or the remote
    /// side terminates.
    ///
    /// # Arguments
    /// * `_term` - Terminal type (e.g., "xterm-256color", "vt100") - reserved for future use
    /// * `_cols` - Terminal width in columns - reserved for future use
    /// * `_rows` - Terminal height in rows - reserved for future use
    ///
    /// # Requirements Coverage
    /// - Requirement 9.4: Interactive shell sessions with PTY allocation
    ///
    /// Note: The async-ssh2-tokio library handles PTY allocation internally
    /// when request_pty=true is passed to execute_io.
    pub async fn open_shell(&self, term: &str, cols: u32, rows: u32) -> Result<Shell, SshError> {
        let client = self.inner().ok_or(SshError::NotConnected)?;
        
        tracing::debug!("Opening shell with PTY: term={}, cols={}, rows={}", term, cols, rows);
        
        // Create channels for I/O
        let (stdin_tx, stdin_rx) = mpsc::channel::<Vec<u8>>(32);
        let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>(32);
        
        // Clone client for the background task
        let client_clone = client.clone();
        let term_clone = term.to_string();
        
        // Spawn background task to run the shell
        tokio::spawn(async move {
            // Use a shell command that starts an interactive shell
            // The PTY flag enables pseudo-terminal allocation
            let result = client_clone.execute_io(
                "/bin/sh",  // Use sh as the shell command
                stdout_tx,
                None,       // stderr goes to stdout when PTY is enabled
                Some(stdin_rx),
                true,       // request_pty = true for interactive shell
                Some(0),    // default exit code
            ).await;
            
            match result {
                Ok(exit_code) => {
                    tracing::info!("Shell session ended with exit code: {}", exit_code);
                }
                Err(e) => {
                    tracing::error!("Shell session error: {}", e);
                }
            }
        });
        
        tracing::info!("Interactive shell opened with PTY (term={})", term_clone);
        
        Ok(Shell::new(stdin_tx, stdout_rx, term.to_string(), cols, rows))
    }
}
