//! SSH-related Tauri commands

use serde::{Deserialize, Serialize};
use tauri::{State, Window};

use crate::error::AppError;
use crate::state::AppState;

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
    
    // Validate auth type
    match request.auth_type.as_str() {
        "password" => {
            if request.password.is_none() {
                return Err(AppError::AuthenticationFailed("Password required".to_string()));
            }
        }
        "key" => {
            if request.key_path.is_none() {
                return Err(AppError::AuthenticationFailed("Key path required".to_string()));
            }
        }
        "agent" => {
            // SSH agent authentication
        }
        _ => return Err(AppError::InvalidAuthMethod),
    }
    
    // Create session
    let session_id = state.create_session(
        request.host.clone(),
        request.username.clone(),
    ).await;
    
    // TODO: Actually connect using russh-ssh library
    // For now, simulate connection
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Mark session as connected
    state.set_session_connected(&session_id).await?;
    
    // Emit connection event
    window.emit("connection-state-changed", serde_json::json!({
        "sessionId": session_id,
        "status": "connected",
        "host": request.host,
        "username": request.username,
    })).ok();
    
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
    
    // TODO: Actually disconnect using russh-ssh library
    
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
    
    // Verify session exists
    let _session = state.get_session(&request.session_id).await
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;
    
    // TODO: Actually execute command using russh-ssh library
    // For now, return mock response
    
    Ok(CommandResponse {
        stdout: format!("Executed: {}", request.command),
        stderr: String::new(),
        exit_code: 0,
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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Start PTY session using russh-ssh library
    // For now, emit a welcome message
    window.emit(&format!("terminal-output-{}", session_id), 
        "Welcome to RUSSH Terminal\r\n$ ".to_string()
    ).ok();
    
    Ok(())
}

/// Send input to terminal
#[tauri::command]
pub async fn terminal_input(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Send input to PTY using russh-ssh library
    // For now, echo the input back
    window.emit(&format!("terminal-output-{}", session_id), data).ok();
    
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
    
    // TODO: Resize PTY using russh-ssh library
    
    Ok(())
}
