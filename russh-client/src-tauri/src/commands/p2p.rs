//! P2P networking Tauri commands

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use image::ImageEncoder;
use tauri::State;

use crate::error::AppError;
use crate::state::{AppState, P2PNodeInfo, P2PPeerInfo};

/// Get local P2P node information
#[tauri::command]
pub async fn p2p_get_node_info(
    state: State<'_, AppState>,
) -> Result<P2PNodeInfo, AppError> {
    tracing::info!("Getting P2P node info");
    Ok(state.get_p2p_node_info().await)
}

/// Connect to a P2P peer
#[tauri::command]
pub async fn p2p_connect(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<P2PPeerInfo, AppError> {
    tracing::info!("Connecting to P2P peer: {}", peer_id);
    
    // TODO: Actually connect using russh-ssh P2P
    // For now, create mock peer info
    let peer_info = P2PPeerInfo {
        peer_id: peer_id.clone(),
        connection_type: "direct".to_string(),
        latency_ms: 50,
        connected_at: Utc::now().to_rfc3339(),
    };
    
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
    
    // TODO: Actually disconnect using russh-ssh P2P
    state.remove_p2p_peer(&peer_id).await
}

/// List connected P2P peers
#[tauri::command]
pub async fn p2p_list_peers(
    state: State<'_, AppState>,
) -> Result<Vec<P2PPeerInfo>, AppError> {
    Ok(state.list_p2p_peers().await)
}

/// Generate QR code for node ID sharing
#[tauri::command]
pub async fn p2p_generate_qr(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let node_info = state.get_p2p_node_info().await;
    
    // Generate QR code
    let qr_data = format!("russh://{}", node_info.node_id);
    
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
