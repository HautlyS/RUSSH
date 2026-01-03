//! SSH-related Tauri commands

use russh_ssh::ssh::{SshClient, SshConfig, AuthMethod, HostKeyCheck};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{Emitter, State, Window};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::{AppState, SessionState};

/// Connection request from frontend
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRequest {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
}

/// Connection response to frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResponse {
    pub session_id: String,
    pub connected: bool,
    pub host: String,
    pub username: String,
}

/// Command request from frontend
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRequest {
    pub session_id: String,
    pub command: String,
    pub timeout_secs: Option<u64>,
}

/// Command response to frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Session info response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub session_id: String,
    pub host: String,
    pub username: String,
    pub status: String,
    pub connected_at: String,
}

/// Connect to SSH server
#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, AppState>,
    window: Window,
    request: ConnectionRequest,
) -> Result<ConnectionResponse, AppError> {
    tracing::info!("Connecting to {}@{}:{}", request.username, request.host, request.port);
    
    // Build auth method
    let auth = match request.auth_type.as_str() {
        "password" => {
            let password = request.password
                .ok_or_else(|| AppError::AuthenticationFailed("Password required".to_string()))?;
            AuthMethod::Password(password)
        }
        "key" => {
            let key_path = request.key_path
                .ok_or_else(|| AppError::AuthenticationFailed("Key path required".to_string()))?;
            AuthMethod::PublicKey {
                key_path: PathBuf::from(key_path),
                passphrase: request.key_passphrase,
            }
        }
        "agent" => AuthMethod::Agent,
        _ => return Err(AppError::InvalidAuthMethod),
    };
    
    // Build SSH config
    let config = SshConfig {
        host: request.host.clone(),
        port: request.port,
        username: request.username.clone(),
        auth,
        timeout: Duration::from_secs(30),
        known_hosts_path: None,
        host_key_check: HostKeyCheck::None, // TODO: Make configurable
    };
    
    // Create and connect SSH client
    let mut client = SshClient::new();
    client.connect(&config).await.map_err(|e| {
        tracing::error!("SSH connection failed: {}", e);
        AppError::ConnectionFailed(e.to_string())
    })?;
    
    // Create session
    let session_id = Uuid::new_v4().to_string();
    let mut session = SessionState::new(
        session_id.clone(),
        request.host.clone(),
        request.username.clone(),
        client,
    );
    session.set_connected();
    
    // Store session
    state.add_session(session_id.clone(), session).await;
    
    // Emit connection event
    window.emit("connection-state-changed", serde_json::json!({
        "sessionId": session_id,
        "status": "connected",
        "host": request.host,
        "username": request.username,
    })).ok();
    
    tracing::info!("SSH connection established: {}", session_id);
    
    Ok(ConnectionResponse {
        session_id,
        connected: true,
        host: request.host,
        username: request.username,
    })
}

/// Disconnect from SSH server
#[tauri::command]
pub async fn ssh_disconnect(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
) -> Result<(), AppError> {
    tracing::info!("Disconnecting session: {}", session_id);
    
    // Get session info before removing
    let session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // Get client and disconnect
    if let Some(client) = state.get_session_client(&session_id).await {
        let mut client = client.lock().await;
        if let Err(e) = client.disconnect().await {
            tracing::warn!("Error during disconnect: {}", e);
        }
    }
    
    // Remove session
    state.remove_session(&session_id).await?;
    
    // Emit disconnection event
    window.emit("connection-state-changed", serde_json::json!({
        "sessionId": session_id,
        "status": "disconnected",
        "host": session.host,
        "username": session.username,
    })).ok();
    
    Ok(())
}

/// Execute command on remote server
#[tauri::command]
pub async fn ssh_execute(
    state: State<'_, AppState>,
    request: CommandRequest,
) -> Result<CommandResponse, AppError> {
    tracing::info!("Executing command on session {}: {}", request.session_id, request.command);
    
    // Get client
    let client = state.get_session_client(&request.session_id).await
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;
    
    let client = client.lock().await;
    
    // Execute command with optional timeout
    let result = if let Some(timeout_secs) = request.timeout_secs {
        client.execute_with_timeout(&request.command, Duration::from_secs(timeout_secs)).await
    } else {
        client.execute(&request.command).await
    };
    
    let result = result.map_err(|e| {
        tracing::error!("Command execution failed: {}", e);
        AppError::InternalError(e.to_string())
    })?;
    
    // Update stats
    state.get_session_mut(&request.session_id, |s| {
        s.increment_commands();
        s.add_bytes_received(result.stdout.len() as u64 + result.stderr.len() as u64);
    }).await;
    
    Ok(CommandResponse {
        stdout: result.stdout_string(),
        stderr: result.stderr_string(),
        exit_code: result.exit_code,
    })
}

/// List active sessions
#[tauri::command]
pub async fn ssh_list_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<SessionResponse>, AppError> {
    let sessions = state.list_sessions().await;
    
    Ok(sessions.into_iter().map(|s| SessionResponse {
        session_id: s.session_id,
        host: s.host,
        username: s.username,
        status: format!("{:?}", s.status).to_lowercase(),
        connected_at: s.connected_at.to_rfc3339(),
    }).collect())
}

/// Start terminal PTY session
#[tauri::command]
pub async fn terminal_start(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
) -> Result<(), AppError> {
    tracing::info!("Starting terminal for session: {}", session_id);
    
    // Get client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // Open shell with PTY
    let mut shell = {
        let client = client.lock().await;
        client.open_shell("xterm-256color", 80, 24).await.map_err(|e| {
            tracing::error!("Failed to open shell: {}", e);
            AppError::InternalError(e.to_string())
        })?
    };
    
    // Create input channel
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(32);
    
    // Spawn task to handle shell I/O
    let win = window.clone();
    let sid = session_id.clone();
    let terminal_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle input from frontend
                Some(data) = input_rx.recv() => {
                    if let Err(e) = shell.write(&data).await {
                        tracing::error!("Failed to write to shell: {}", e);
                        break;
                    }
                }
                // Read output from shell
                output = shell.read() => {
                    match output {
                        Some(bytes) if !bytes.is_empty() => {
                            let text = String::from_utf8_lossy(&bytes).to_string();
                            if win.emit(&format!("terminal-output-{}", sid), &text).is_err() {
                                break;
                            }
                        }
                        None => {
                            tracing::info!("Shell closed for session: {}", sid);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        tracing::info!("Terminal task ended for session: {}", sid);
    });
    
    // Store terminal handles in session
    state.get_session_mut(&session_id, |s| {
        s.terminal_task = Some(terminal_task);
        s.terminal_input_tx = Some(input_tx);
    }).await;
    
    Ok(())
}

/// Send input to terminal
#[tauri::command]
pub async fn terminal_input(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
    // Get input sender from session
    let tx = state.get_terminal_input_tx(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // Send input to terminal task
    tx.send(data.into_bytes()).await
        .map_err(|e| AppError::InternalError(format!("Failed to send input: {}", e)))?;
    
    Ok(())
}

/// Resize terminal
#[tauri::command]
pub async fn terminal_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    tracing::debug!("Resizing terminal {} to {}x{}", session_id, cols, rows);
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Implement PTY resize - requires extending Shell with resize capability
    // The async-ssh2-tokio library would need to support window-change requests
    
    Ok(())
}
