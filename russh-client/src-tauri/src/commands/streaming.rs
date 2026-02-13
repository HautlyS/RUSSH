//! Video streaming Tauri commands

use russh_ssh::streaming::{PlaybackState, StreamRoom, StreamSession, StreamSource};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State, Window};

use crate::error::AppError;
use crate::state::AppState;

/// Stream room response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamRoomResponse {
    pub room_id: String,
    pub name: String,
    pub host_id: String,
    pub source: StreamSourceResponse,
    pub playback: PlaybackStateResponse,
    pub peers: Vec<String>,
    pub share_link: String,
}

/// Stream source response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum StreamSourceResponse {
    Url {
        url: String,
    },
    LocalFile {
        path: String,
        size: u64,
    },
    P2PFile {
        host_id: String,
        file_id: String,
        size: u64,
    },
}

/// Playback state response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateResponse {
    pub playing: bool,
    pub position: f64,
    pub speed: f64,
    pub sync_time: i64,
}

impl From<PlaybackState> for PlaybackStateResponse {
    fn from(state: PlaybackState) -> Self {
        Self {
            playing: state.playing,
            position: state.position,
            speed: state.speed,
            sync_time: state.sync_time,
        }
    }
}

impl From<StreamSource> for StreamSourceResponse {
    fn from(source: StreamSource) -> Self {
        match source {
            StreamSource::Url { url } => StreamSourceResponse::Url { url },
            StreamSource::LocalFile { path, size } => {
                StreamSourceResponse::LocalFile { path, size }
            }
            StreamSource::P2PFile {
                host_id,
                file_id,
                size,
            } => StreamSourceResponse::P2PFile {
                host_id,
                file_id,
                size,
            },
        }
    }
}

/// Create stream request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStreamRequest {
    pub name: String,
    pub source_type: String,
    pub url: Option<String>,
    pub file_path: Option<String>,
}

/// Sync event request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEventRequest {
    pub room_id: String,
    pub event_type: String,
    pub position: Option<f64>,
    pub speed: Option<f64>,
}

/// Create a new stream room
#[tauri::command]
pub async fn stream_create_room(
    state: State<'_, AppState>,
    window: Window,
    request: CreateStreamRequest,
) -> Result<StreamRoomResponse, AppError> {
    tracing::info!("Creating stream room: {}", request.name);

    // Get node ID for host
    let host_id = if let Some((endpoint, _)) = state.get_p2p_state().await {
        endpoint.node_id().to_string()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    // Build source
    let source = match request.source_type.as_str() {
        "url" => {
            let url = request.url.ok_or_else(|| {
                AppError::InternalError("URL required for URL source".to_string())
            })?;
            StreamSource::Url { url }
        }
        "file" => {
            let path = request.file_path.ok_or_else(|| {
                AppError::InternalError("File path required for file source".to_string())
            })?;
            let metadata = std::fs::metadata(&path).map_err(|e| {
                AppError::FileOperationFailed(format!("Failed to read file: {}", e))
            })?;
            StreamSource::LocalFile {
                path,
                size: metadata.len(),
            }
        }
        _ => return Err(AppError::InternalError("Invalid source type".to_string())),
    };

    // Create session
    let session = StreamSession::create_room(request.name, source, host_id);
    let room = session.room().await;
    let share_link = session.share_link().await;
    let room_id = room.room_id.clone();

    // Store session
    state
        .add_stream_session(room_id.clone(), Arc::new(session))
        .await;

    // Start event listener
    let win = window.clone();
    let rid = room_id.clone();
    if let Some(session) = state.get_stream_session(&room_id).await {
        let mut rx = session.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let event_json = serde_json::to_value(&event).unwrap_or_default();
                win.emit(&format!("stream-event-{}", rid), event_json).ok();
            }
        });
    }

    Ok(StreamRoomResponse {
        room_id: room.room_id,
        name: room.name,
        host_id: room.host_id,
        source: room.source.into(),
        playback: room.playback.into(),
        peers: room.peers,
        share_link,
    })
}

/// Join an existing stream room
#[tauri::command]
pub async fn stream_join_room(
    state: State<'_, AppState>,
    window: Window,
    room_id: String,
    host_id: String,
) -> Result<StreamRoomResponse, AppError> {
    tracing::info!("Joining stream room: {} (host: {})", room_id, host_id);

    // Connect to host via P2P
    let (_, manager) = state
        .get_p2p_state()
        .await
        .ok_or_else(|| AppError::P2PConnectionFailed("P2P not initialized".to_string()))?;

    let node_id: russh_ssh::NodeId = host_id
        .parse()
        .map_err(|e| AppError::P2PConnectionFailed(format!("Invalid host ID: {}", e)))?;

    // Connect to host
    let _connection = manager
        .connect(node_id)
        .await
        .map_err(|e| AppError::P2PConnectionFailed(e.to_string()))?;

    // TODO: Request room info from host via P2P stream
    // For now, create a placeholder room
    let room = StreamRoom {
        room_id: room_id.clone(),
        name: "Joined Room".to_string(),
        host_id: host_id.clone(),
        source: StreamSource::Url { url: String::new() },
        playback: PlaybackState::default(),
        peers: vec![],
        created_at: chrono::Utc::now().timestamp(),
    };

    let session = StreamSession::join_room(room.clone()).with_p2p(manager);
    let share_link = session.share_link().await;

    // Store session
    state
        .add_stream_session(room_id.clone(), Arc::new(session))
        .await;

    // Start event listener
    let win = window.clone();
    let rid = room_id.clone();
    if let Some(session) = state.get_stream_session(&room_id).await {
        let mut rx = session.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let event_json = serde_json::to_value(&event).unwrap_or_default();
                win.emit(&format!("stream-event-{}", rid), event_json).ok();
            }
        });
    }

    Ok(StreamRoomResponse {
        room_id: room.room_id,
        name: room.name,
        host_id: room.host_id,
        source: room.source.into(),
        playback: room.playback.into(),
        peers: room.peers,
        share_link,
    })
}

/// Leave a stream room
#[tauri::command]
pub async fn stream_leave_room(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<(), AppError> {
    tracing::info!("Leaving stream room: {}", room_id);
    state.remove_stream_session(&room_id).await;
    Ok(())
}

/// Get stream room info
#[tauri::command]
pub async fn stream_get_room(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<StreamRoomResponse, AppError> {
    let session = state
        .get_stream_session(&room_id)
        .await
        .ok_or_else(|| AppError::InternalError("Room not found".to_string()))?;

    let room = session.room().await;
    let share_link = session.share_link().await;

    Ok(StreamRoomResponse {
        room_id: room.room_id,
        name: room.name,
        host_id: room.host_id,
        source: room.source.into(),
        playback: room.playback.into(),
        peers: room.peers,
        share_link,
    })
}

/// Send sync event (play/pause/seek)
#[tauri::command]
pub async fn stream_sync(
    state: State<'_, AppState>,
    request: SyncEventRequest,
) -> Result<(), AppError> {
    let session = state
        .get_stream_session(&request.room_id)
        .await
        .ok_or_else(|| AppError::InternalError("Room not found".to_string()))?;

    match request.event_type.as_str() {
        "play" => {
            session
                .play()
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }
        "pause" => {
            session
                .pause()
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }
        "seek" => {
            let position = request.position.unwrap_or(0.0);
            session
                .seek(position)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }
        "speed" => {
            let speed = request.speed.unwrap_or(1.0);
            session
                .set_speed(speed)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }
        _ => return Err(AppError::InternalError("Invalid event type".to_string())),
    }

    Ok(())
}

/// Update playback position (called periodically)
#[tauri::command]
pub async fn stream_update_position(
    state: State<'_, AppState>,
    room_id: String,
    position: f64,
) -> Result<(), AppError> {
    if let Some(session) = state.get_stream_session(&room_id).await {
        session.update_position(position).await;
    }
    Ok(())
}

/// Get expected position (for sync correction)
#[tauri::command]
pub async fn stream_get_expected_position(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<f64, AppError> {
    let session = state
        .get_stream_session(&room_id)
        .await
        .ok_or_else(|| AppError::InternalError("Room not found".to_string()))?;

    Ok(session.expected_position().await)
}
