//! File transfer Tauri commands

use serde::{Deserialize, Serialize};
use tauri::{Emitter, State, Window};
use uuid::Uuid;
use std::path::Path;

use crate::error::AppError;
use crate::state::AppState;

/// File entry information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub modified: String,
    pub owner: String,
}

/// Transfer progress information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgress {
    pub transfer_id: String,
    pub filename: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
    pub status: String,
}

/// List directory contents
#[tauri::command]
pub async fn file_list(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<Vec<FileEntry>, AppError> {
    tracing::info!("Listing directory {} for session {}", path, session_id);
    
    // Get SSH client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let client = client.lock().await;
    
    // List directory using SFTP
    let entries = client.list_directory(&path).await.map_err(|e| {
        tracing::error!("Failed to list directory: {}", e);
        AppError::FileOperationFailed(e.to_string())
    })?;
    
    // Convert to frontend format
    Ok(entries.into_iter().map(|e| FileEntry {
        name: e.name,
        path: e.path,
        is_dir: e.is_dir,
        size: e.size,
        permissions: e.permissions,
        modified: e.modified,
        owner: e.owner,
    }).collect())
}

/// Upload file to remote server
#[tauri::command]
pub async fn file_upload(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
    local_path: String,
    remote_path: String,
) -> Result<String, AppError> {
    tracing::info!("Uploading {} to {} for session {}", local_path, remote_path, session_id);
    
    // Get SSH client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let transfer_id = Uuid::new_v4().to_string();
    let filename = Path::new(&local_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| local_path.clone());
    
    // Read local file
    let data = tokio::fs::read(&local_path).await.map_err(|e| {
        AppError::FileOperationFailed(format!("Failed to read local file: {}", e))
    })?;
    
    let total_bytes = data.len() as u64;
    let tid = transfer_id.clone();
    let fname = filename.clone();
    let win = window.clone();
    
    // Emit initial progress
    win.emit("transfer-progress", TransferProgress {
        transfer_id: tid.clone(),
        filename: fname.clone(),
        bytes_transferred: 0,
        total_bytes,
        speed_bps: 0,
        eta_seconds: 0,
        status: "active".to_string(),
    }).ok();
    
    // Upload file
    {
        let client = client.lock().await;
        client.write_file(&remote_path, &data).await.map_err(|e| {
            tracing::error!("Failed to upload file: {}", e);
            AppError::TransferFailed(e.to_string())
        })?;
    }
    
    // Emit completion
    window.emit("transfer-progress", TransferProgress {
        transfer_id: tid,
        filename: fname,
        bytes_transferred: total_bytes,
        total_bytes,
        speed_bps: 0,
        eta_seconds: 0,
        status: "completed".to_string(),
    }).ok();
    
    Ok(transfer_id)
}

/// Download file from remote server
#[tauri::command]
pub async fn file_download(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
    remote_path: String,
    local_path: String,
) -> Result<String, AppError> {
    tracing::info!("Downloading {} to {} for session {}", remote_path, local_path, session_id);
    
    // Get SSH client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let transfer_id = Uuid::new_v4().to_string();
    let filename = Path::new(&remote_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| remote_path.clone());
    
    // Get file size first
    let total_bytes = {
        let client = client.lock().await;
        client.file_size(&remote_path).await.unwrap_or(0)
    };
    
    let tid = transfer_id.clone();
    let fname = filename.clone();
    let win = window.clone();
    
    // Emit initial progress
    win.emit("transfer-progress", TransferProgress {
        transfer_id: tid.clone(),
        filename: fname.clone(),
        bytes_transferred: 0,
        total_bytes,
        speed_bps: 0,
        eta_seconds: 0,
        status: "active".to_string(),
    }).ok();
    
    // Download file
    let data = {
        let client = client.lock().await;
        client.read_file(&remote_path).await.map_err(|e| {
            tracing::error!("Failed to download file: {}", e);
            AppError::TransferFailed(e.to_string())
        })?
    };
    
    // Write to local file
    tokio::fs::write(&local_path, &data).await.map_err(|e| {
        AppError::FileOperationFailed(format!("Failed to write local file: {}", e))
    })?;
    
    // Emit completion
    window.emit("transfer-progress", TransferProgress {
        transfer_id: tid,
        filename: fname,
        bytes_transferred: data.len() as u64,
        total_bytes: data.len() as u64,
        speed_bps: 0,
        eta_seconds: 0,
        status: "completed".to_string(),
    }).ok();
    
    Ok(transfer_id)
}

/// Delete file or directory
#[tauri::command]
pub async fn file_delete(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), AppError> {
    tracing::info!("Deleting {} for session {}", path, session_id);
    
    // Get SSH client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let client = client.lock().await;
    
    // Check if it's a directory
    let is_dir = client.stat_path(&path).await
        .map(|s| s.is_dir)
        .unwrap_or(false);
    
    client.delete_path(&path, is_dir).await.map_err(|e| {
        tracing::error!("Failed to delete: {}", e);
        AppError::FileOperationFailed(e.to_string())
    })?;
    
    Ok(())
}

/// Rename file or directory
#[tauri::command]
pub async fn file_rename(
    state: State<'_, AppState>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), AppError> {
    tracing::info!("Renaming {} to {} for session {}", old_path, new_path, session_id);
    
    // Get SSH client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let client = client.lock().await;
    
    client.rename_path(&old_path, &new_path).await.map_err(|e| {
        tracing::error!("Failed to rename: {}", e);
        AppError::FileOperationFailed(e.to_string())
    })?;
    
    Ok(())
}

/// Create directory
#[tauri::command]
pub async fn file_mkdir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), AppError> {
    tracing::info!("Creating directory {} for session {}", path, session_id);
    
    // Get SSH client
    let client = state.get_session_client(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let client = client.lock().await;
    
    client.create_directory(&path).await.map_err(|e| {
        tracing::error!("Failed to create directory: {}", e);
        AppError::FileOperationFailed(e.to_string())
    })?;
    
    Ok(())
}
