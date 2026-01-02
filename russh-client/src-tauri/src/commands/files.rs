//! File transfer Tauri commands

use serde::{Deserialize, Serialize};
use tauri::{State, Window};
use uuid::Uuid;

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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Actually list directory using russh-ssh SFTP
    // For now, return mock data
    Ok(vec![
        FileEntry {
            name: "..".to_string(),
            path: get_parent_path(&path),
            is_dir: true,
            size: 0,
            permissions: "drwxr-xr-x".to_string(),
            modified: "2024-01-01T00:00:00Z".to_string(),
            owner: "root".to_string(),
        },
        FileEntry {
            name: "home".to_string(),
            path: format!("{}/home", path.trim_end_matches('/')),
            is_dir: true,
            size: 4096,
            permissions: "drwxr-xr-x".to_string(),
            modified: "2024-01-01T00:00:00Z".to_string(),
            owner: "root".to_string(),
        },
        FileEntry {
            name: "etc".to_string(),
            path: format!("{}/etc", path.trim_end_matches('/')),
            is_dir: true,
            size: 4096,
            permissions: "drwxr-xr-x".to_string(),
            modified: "2024-01-01T00:00:00Z".to_string(),
            owner: "root".to_string(),
        },
        FileEntry {
            name: "README.md".to_string(),
            path: format!("{}/README.md", path.trim_end_matches('/')),
            is_dir: false,
            size: 1024,
            permissions: "-rw-r--r--".to_string(),
            modified: "2024-01-01T00:00:00Z".to_string(),
            owner: "user".to_string(),
        },
    ])
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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let transfer_id = Uuid::new_v4().to_string();
    let filename = local_path.split('/').last()
        .unwrap_or(&local_path)
        .to_string();
    
    // TODO: Actually upload file using russh-ssh SFTP
    // For now, simulate progress
    let tid = transfer_id.clone();
    let win = window.clone();
    let fname = filename.clone();
    
    tokio::spawn(async move {
        let total_bytes = 1024 * 1024; // 1MB mock file
        let mut transferred = 0u64;
        
        while transferred < total_bytes {
            transferred += 102400; // 100KB chunks
            if transferred > total_bytes {
                transferred = total_bytes;
            }
            
            let progress = TransferProgress {
                transfer_id: tid.clone(),
                filename: fname.clone(),
                bytes_transferred: transferred,
                total_bytes,
                speed_bps: 1024 * 1024, // 1MB/s
                eta_seconds: (total_bytes - transferred) / (1024 * 1024),
                status: if transferred >= total_bytes { "completed" } else { "active" }.to_string(),
            };
            
            win.emit("transfer-progress", &progress).ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
    
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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    let transfer_id = Uuid::new_v4().to_string();
    let filename = remote_path.split('/').last()
        .unwrap_or(&remote_path)
        .to_string();
    
    // TODO: Actually download file using russh-ssh SFTP
    // For now, simulate progress
    let tid = transfer_id.clone();
    let win = window.clone();
    let fname = filename.clone();
    
    tokio::spawn(async move {
        let total_bytes = 1024 * 1024; // 1MB mock file
        let mut transferred = 0u64;
        
        while transferred < total_bytes {
            transferred += 102400; // 100KB chunks
            if transferred > total_bytes {
                transferred = total_bytes;
            }
            
            let progress = TransferProgress {
                transfer_id: tid.clone(),
                filename: fname.clone(),
                bytes_transferred: transferred,
                total_bytes,
                speed_bps: 1024 * 1024, // 1MB/s
                eta_seconds: (total_bytes - transferred) / (1024 * 1024),
                status: if transferred >= total_bytes { "completed" } else { "active" }.to_string(),
            };
            
            win.emit("transfer-progress", &progress).ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
    
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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Actually delete file using russh-ssh SFTP
    
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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Actually rename file using russh-ssh SFTP
    
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
    
    // Verify session exists
    let _session = state.get_session(&session_id).await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    // TODO: Actually create directory using russh-ssh SFTP
    
    Ok(())
}

/// Get parent path
fn get_parent_path(path: &str) -> String {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }
    
    match path.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => path[..idx].to_string(),
        None => "/".to_string(),
    }
}
