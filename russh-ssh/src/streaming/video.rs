//! P2P Video Streaming Module
//!
//! Provides synchronized video streaming over P2P connections.
//! Uses stream-download-rs for efficient streaming with seeking support.

use crate::error::StreamError;
use crate::p2p::P2PConnectionManager;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Stream room for synchronized playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRoom {
    /// Unique room ID
    pub room_id: String,
    /// Room name
    pub name: String,
    /// Host node ID
    pub host_id: String,
    /// Current media source
    pub source: StreamSource,
    /// Current playback state
    pub playback: PlaybackState,
    /// Connected peers
    pub peers: Vec<String>,
    /// Created timestamp
    pub created_at: i64,
}

/// Stream source types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamSource {
    /// HTTP/HTTPS URL
    Url { url: String },
    /// Local file (host only)
    LocalFile { path: String, size: u64 },
    /// P2P shared file
    P2PFile {
        host_id: String,
        file_id: String,
        size: u64,
    },
}

/// Playback state for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    /// Is playing
    pub playing: bool,
    /// Current position in seconds
    pub position: f64,
    /// Playback speed (1.0 = normal)
    pub speed: f64,
    /// Last sync timestamp (Unix ms)
    pub sync_time: i64,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            playing: false,
            position: 0.0,
            speed: 1.0,
            sync_time: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// Sync event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncEvent {
    /// Play event
    Play { position: f64 },
    /// Pause event
    Pause { position: f64 },
    /// Seek event
    Seek { position: f64 },
    /// Speed change
    Speed { speed: f64 },
    /// Peer joined
    PeerJoined { peer_id: String },
    /// Peer left
    PeerLeft { peer_id: String },
    /// Source changed
    SourceChanged { source: StreamSource },
    /// Request sync (from peer)
    RequestSync,
    /// Full state sync (from host)
    StateSync { state: PlaybackState },
}

/// Stream session manager
pub struct StreamSession {
    /// Session ID
    pub session_id: String,
    /// Room info
    room: Arc<RwLock<StreamRoom>>,
    /// Is host
    is_host: bool,
    /// Event sender
    event_tx: broadcast::Sender<SyncEvent>,
    /// P2P connection manager
    p2p_manager: Option<Arc<P2PConnectionManager>>,
}

impl StreamSession {
    /// Create a new stream session as host
    pub fn create_room(name: String, source: StreamSource, host_id: String) -> Self {
        let room_id = Uuid::new_v4().to_string();
        let (event_tx, _) = broadcast::channel(100);

        let room = StreamRoom {
            room_id: room_id.clone(),
            name,
            host_id,
            source,
            playback: PlaybackState::default(),
            peers: vec![],
            created_at: chrono::Utc::now().timestamp(),
        };

        Self {
            session_id: room_id,
            room: Arc::new(RwLock::new(room)),
            is_host: true,
            event_tx,
            p2p_manager: None,
        }
    }

    /// Join an existing room
    pub fn join_room(room: StreamRoom) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let session_id = room.room_id.clone();

        Self {
            session_id,
            room: Arc::new(RwLock::new(room)),
            is_host: false,
            event_tx,
            p2p_manager: None,
        }
    }

    /// Set P2P manager for peer communication
    pub fn with_p2p(mut self, manager: Arc<P2PConnectionManager>) -> Self {
        self.p2p_manager = Some(manager);
        self
    }

    /// Get room info
    pub async fn room(&self) -> StreamRoom {
        self.room.read().await.clone()
    }

    /// Get share link
    pub async fn share_link(&self) -> String {
        let room = self.room.read().await;
        format!("russh://stream/{}?host={}", room.room_id, room.host_id)
    }

    /// Subscribe to sync events
    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_tx.subscribe()
    }

    /// Play
    pub async fn play(&self) -> Result<(), StreamError> {
        let mut room = self.room.write().await;
        room.playback.playing = true;
        room.playback.sync_time = chrono::Utc::now().timestamp_millis();

        let event = SyncEvent::Play {
            position: room.playback.position,
        };
        self.broadcast_event(event).await
    }

    /// Pause
    pub async fn pause(&self) -> Result<(), StreamError> {
        let mut room = self.room.write().await;
        room.playback.playing = false;
        room.playback.sync_time = chrono::Utc::now().timestamp_millis();

        let event = SyncEvent::Pause {
            position: room.playback.position,
        };
        self.broadcast_event(event).await
    }

    /// Seek to position
    pub async fn seek(&self, position: f64) -> Result<(), StreamError> {
        let mut room = self.room.write().await;
        room.playback.position = position;
        room.playback.sync_time = chrono::Utc::now().timestamp_millis();

        let event = SyncEvent::Seek { position };
        self.broadcast_event(event).await
    }

    /// Update position (called periodically during playback)
    pub async fn update_position(&self, position: f64) {
        let mut room = self.room.write().await;
        room.playback.position = position;
    }

    /// Set playback speed
    pub async fn set_speed(&self, speed: f64) -> Result<(), StreamError> {
        let mut room = self.room.write().await;
        room.playback.speed = speed;
        room.playback.sync_time = chrono::Utc::now().timestamp_millis();

        let event = SyncEvent::Speed { speed };
        self.broadcast_event(event).await
    }

    /// Change source
    pub async fn change_source(&self, source: StreamSource) -> Result<(), StreamError> {
        if !self.is_host {
            return Err(StreamError::NotFound(
                "Only host can change source".to_string(),
            ));
        }

        let mut room = self.room.write().await;
        room.source = source.clone();
        room.playback = PlaybackState::default();

        let event = SyncEvent::SourceChanged { source };
        self.broadcast_event(event).await
    }

    /// Handle incoming sync event
    pub async fn handle_event(&self, event: SyncEvent) -> Result<(), StreamError> {
        match &event {
            SyncEvent::Play { position } => {
                let mut room = self.room.write().await;
                room.playback.playing = true;
                room.playback.position = *position;
                room.playback.sync_time = chrono::Utc::now().timestamp_millis();
            }
            SyncEvent::Pause { position } => {
                let mut room = self.room.write().await;
                room.playback.playing = false;
                room.playback.position = *position;
                room.playback.sync_time = chrono::Utc::now().timestamp_millis();
            }
            SyncEvent::Seek { position } => {
                let mut room = self.room.write().await;
                room.playback.position = *position;
                room.playback.sync_time = chrono::Utc::now().timestamp_millis();
            }
            SyncEvent::Speed { speed } => {
                let mut room = self.room.write().await;
                room.playback.speed = *speed;
            }
            SyncEvent::PeerJoined { peer_id } => {
                let mut room = self.room.write().await;
                if !room.peers.contains(peer_id) {
                    room.peers.push(peer_id.clone());
                }
            }
            SyncEvent::PeerLeft { peer_id } => {
                let mut room = self.room.write().await;
                room.peers.retain(|p| p != peer_id);
            }
            SyncEvent::SourceChanged { source } => {
                let mut room = self.room.write().await;
                room.source = source.clone();
                room.playback = PlaybackState::default();
            }
            SyncEvent::RequestSync => {
                if self.is_host {
                    let room = self.room.read().await;
                    let sync_event = SyncEvent::StateSync {
                        state: room.playback.clone(),
                    };
                    self.broadcast_event(sync_event).await?;
                }
            }
            SyncEvent::StateSync { state } => {
                let mut room = self.room.write().await;
                room.playback = state.clone();
            }
        }

        // Re-broadcast to local subscribers
        let _ = self.event_tx.send(event);
        Ok(())
    }

    /// Broadcast event to all peers
    async fn broadcast_event(&self, event: SyncEvent) -> Result<(), StreamError> {
        // Send to local subscribers
        let _ = self.event_tx.send(event.clone());

        // TODO: Send to P2P peers via connection manager
        // This would serialize the event and send over QUIC streams

        Ok(())
    }

    /// Get current playback state
    pub async fn playback_state(&self) -> PlaybackState {
        self.room.read().await.playback.clone()
    }

    /// Calculate expected position based on sync time
    pub async fn expected_position(&self) -> f64 {
        let room = self.room.read().await;
        if !room.playback.playing {
            return room.playback.position;
        }

        let now = chrono::Utc::now().timestamp_millis();
        let elapsed_ms = (now - room.playback.sync_time) as f64;
        let elapsed_secs = elapsed_ms / 1000.0;

        room.playback.position + (elapsed_secs * room.playback.speed)
    }
}

/// HTTP video stream using stream-download
pub struct HttpVideoStream {
    /// Stream download reader
    reader: stream_download::StreamDownload<stream_download::storage::temp::TempStorageProvider>,
    /// Content length
    content_length: Option<u64>,
    /// Content type
    content_type: Option<String>,
}

impl HttpVideoStream {
    /// Create a new HTTP video stream
    pub async fn new(url: &str) -> Result<Self, StreamError> {
        use stream_download::storage::temp::TempStorageProvider;
        use stream_download::{Settings, StreamDownload};

        let url = url
            .parse()
            .map_err(|e| StreamError::NotFound(format!("Invalid URL: {}", e)))?;

        let reader =
            StreamDownload::new_http(url, TempStorageProvider::default(), Settings::default())
                .await
                .map_err(|e| StreamError::NotFound(format!("Failed to create stream: {:?}", e)))?;

        Ok(Self {
            reader,
            content_length: None,
            content_type: None,
        })
    }

    /// Get content length
    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    /// Get content type
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}

impl Read for HttpVideoStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Seek for HttpVideoStream {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_state_default() {
        let state = PlaybackState::default();
        assert!(!state.playing);
        assert_eq!(state.position, 0.0);
        assert_eq!(state.speed, 1.0);
    }

    #[test]
    fn stream_room_creation() {
        let source = StreamSource::Url {
            url: "https://example.com/video.mp4".to_string(),
        };
        let session =
            StreamSession::create_room("Test Room".to_string(), source, "host123".to_string());

        assert!(session.is_host);
        assert!(!session.session_id.is_empty());
    }

    #[tokio::test]
    async fn stream_session_playback() {
        let source = StreamSource::Url {
            url: "https://example.com/video.mp4".to_string(),
        };
        let session = StreamSession::create_room("Test".to_string(), source, "host".to_string());

        // Subscribe before events
        let mut rx = session.subscribe();

        session.play().await.unwrap();
        let state = session.playback_state().await;
        assert!(state.playing);

        session.seek(30.0).await.unwrap();
        let state = session.playback_state().await;
        assert_eq!(state.position, 30.0);

        session.pause().await.unwrap();
        let state = session.playback_state().await;
        assert!(!state.playing);
    }

    #[test]
    fn sync_event_serialization() {
        let event = SyncEvent::Play { position: 10.5 };
        let json = serde_json::to_string(&event).unwrap();
        let restored: SyncEvent = serde_json::from_str(&json).unwrap();

        match restored {
            SyncEvent::Play { position } => assert_eq!(position, 10.5),
            _ => panic!("Wrong event type"),
        }
    }
}
