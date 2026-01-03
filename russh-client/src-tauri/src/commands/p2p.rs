//! P2P networking Tauri commands

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use image::ImageEncoder;
use russh_ssh::p2p::{P2PConfig, P2PEndpoint, P2PConnectionManager};
use russh_ssh::NodeId;
use std::sync::Arc;
use tauri::State;

use crate::error::AppError;
use crate::state::{AppState, P2PNodeInfo, P2PPeerInfo};

/// Initialize P2P endpoint if not already initialized
async fn ensure_p2p_initialized(state: &AppState) -> Result<(Arc<P2PEndpoint>, Arc<P2PConnectionManager>), AppError> {
    // Check if already initialized
    if let Some((endpoint, manager)) = state.get_p2p_state().await {
        return Ok((endpoint, manager));
    }
    
    // Initialize P2P endpoint
    tracing::info!("Initializing P2P endpoint");
    let config = P2PConfig::default();
    let endpoint = P2PEndpoint::bind(config).await.map_err(|e| {
        tracing::error!("Failed to initialize P2P: {}", e);
        AppError::P2PConnectionFailed(e.to_string())
    })?;
    
    // Wait for endpoint to be online
    endpoint.wait_online().await;
    
    let endpoint = Arc::new(endpoint);
    let manager = Arc::new(P2PConnectionManager::new(endpoint.clone()));
    
    // Store in state
    state.set_p2p_state(endpoint.clone(), manager.clone()).await;
    
    tracing::info!("P2P endpoint initialized: {}", endpoint.node_id());
    
    Ok((endpoint, manager))
}

/// Get local P2P node information
#[tauri::command]
pub async fn p2p_get_node_info(
    state: State<'_, AppState>,
) -> Result<P2PNodeInfo, AppError> {
    tracing::info!("Getting P2P node info");
    
    let (endpoint, _) = ensure_p2p_initialized(&state).await?;
    
    let node_id = endpoint.node_id().to_string();
    let relay_url = endpoint.relay_url().map(|u| u.to_string());
    let direct_addresses: Vec<String> = endpoint.direct_addresses()
        .iter()
        .map(|a| a.to_string())
        .collect();
    let is_online = endpoint.is_online().await;
    
    Ok(P2PNodeInfo {
        node_id,
        relay_url,
        direct_addresses,
        is_online,
    })
}

/// Connect to a P2P peer
#[tauri::command]
pub async fn p2p_connect(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<P2PPeerInfo, AppError> {
    tracing::info!("Connecting to P2P peer: {}", peer_id);
    
    let (_, manager) = ensure_p2p_initialized(&state).await?;
    
    // Parse peer ID
    let node_id: NodeId = peer_id.parse().map_err(|e| {
        AppError::P2PConnectionFailed(format!("Invalid peer ID: {}", e))
    })?;
    
    // Connect to peer
    let connection = manager.connect(node_id).await.map_err(|e| {
        tracing::error!("Failed to connect to peer: {}", e);
        AppError::P2PConnectionFailed(e.to_string())
    })?;
    
    // Get connection info
    let info = connection.info().await;
    
    let peer_info = P2PPeerInfo {
        peer_id: peer_id.clone(),
        connection_type: info.connection_type.to_string(),
        latency_ms: info.latency.map(|d| d.as_millis() as u64).unwrap_or(0),
        connected_at: Utc::now().to_rfc3339(),
    };
    
    // Store in app state for tracking
    state.add_p2p_peer(peer_id, peer_info.clone()).await;
    
    Ok(peer_info)
}

/// Disconnect from a P2P peer
#[tauri::command]
pub async fn p2p_disconnect(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<(), AppError> {
    tracing::info!("Disconnecting from P2P peer: {}", peer_id);
    
    if let Some((_, manager)) = state.get_p2p_state().await {
        // Parse peer ID
        if let Ok(node_id) = peer_id.parse::<NodeId>() {
            manager.disconnect(&node_id).await;
        }
    }
    
    // Remove from app state
    state.remove_p2p_peer(&peer_id).await.ok();
    
    Ok(())
}

/// List connected P2P peers
#[tauri::command]
pub async fn p2p_list_peers(
    state: State<'_, AppState>,
) -> Result<Vec<P2PPeerInfo>, AppError> {
    // Get peers from connection manager if available
    if let Some((_, manager)) = state.get_p2p_state().await {
        let peer_ids = manager.connected_peers().await;
        let mut peers = Vec::new();
        
        for node_id in peer_ids {
            if let Some(info) = manager.connection_info(&node_id).await {
                peers.push(P2PPeerInfo {
                    peer_id: node_id.to_string(),
                    connection_type: info.connection_type.to_string(),
                    latency_ms: info.latency.map(|d| d.as_millis() as u64).unwrap_or(0),
                    connected_at: Utc::now().to_rfc3339(), // TODO: Store actual connect time
                });
            }
        }
        
        return Ok(peers);
    }
    
    // Fall back to stored peers
    Ok(state.list_p2p_peers().await)
}

/// Generate QR code for node ID sharing
#[tauri::command]
pub async fn p2p_generate_qr(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let (endpoint, _) = ensure_p2p_initialized(&state).await?;
    
    // Get node address for sharing
    let node_addr = endpoint.node_addr().await.map_err(|e| {
        AppError::InternalError(format!("Failed to get node address: {}", e))
    })?;
    
    // Create shareable connection string
    // Format: russh://node_id?relay=url&addr=ip:port
    let mut qr_data = format!("russh://{}", node_addr.node_id);
    
    if let Some(relay) = node_addr.relay_url {
        qr_data.push_str(&format!("?relay={}", relay));
    }
    
    // Generate QR code
    let qr = qrcode::QrCode::new(qr_data.as_bytes())
        .map_err(|e| AppError::InternalError(format!("Failed to generate QR code: {}", e)))?;
    
    // Render to image
    let image = qr.render::<image::Luma<u8>>()
        .min_dimensions(200, 200)
        .build();
    
    // Convert to PNG bytes
    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder.write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::L8,
    ).map_err(|e| AppError::InternalError(format!("Failed to encode QR code: {}", e)))?;
    
    // Return as base64 data URL
    let base64_data = BASE64.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{}", base64_data))
}
