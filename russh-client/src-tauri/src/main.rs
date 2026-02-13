//! RUSSH Client - Cross-Device SSH Client
//!
//! Main entry point for the Tauri application.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod error;
mod state;

use state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Set up panic handler for better error reporting
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        eprintln!("PANIC at {}: {}", location, message);
        tracing::error!("PANIC at {}: {}", location, message);
    }));

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting RUSSH Client");

    let state = AppState::new();
    let state_clone = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // SSH commands
            commands::ssh::ssh_connect,
            commands::ssh::ssh_disconnect,
            commands::ssh::ssh_execute,
            commands::ssh::ssh_list_sessions,
            commands::ssh::terminal_start,
            commands::ssh::terminal_input,
            commands::ssh::terminal_resize,
            // Profile commands
            commands::profiles::profile_create,
            commands::profiles::profile_update,
            commands::profiles::profile_delete,
            commands::profiles::profile_list,
            commands::profiles::profile_export,
            commands::profiles::profile_import,
            // File commands
            commands::files::file_list,
            commands::files::file_upload,
            commands::files::file_download,
            commands::files::file_delete,
            commands::files::file_rename,
            commands::files::file_mkdir,
            // P2P commands
            commands::p2p::p2p_get_node_info,
            commands::p2p::p2p_connect,
            commands::p2p::p2p_disconnect,
            commands::p2p::p2p_list_peers,
            commands::p2p::p2p_generate_qr,
            // Settings commands
            commands::settings::settings_load,
            commands::settings::settings_save,
            // Streaming commands
            commands::streaming::stream_create_room,
            commands::streaming::stream_join_room,
            commands::streaming::stream_leave_room,
            commands::streaming::stream_get_room,
            commands::streaming::stream_sync,
            commands::streaming::stream_update_position,
            commands::streaming::stream_get_expected_position,
        ])
        .setup(move |_app| {
            tauri::async_runtime::spawn(async move {
                if let Err(e) = state_clone.load_profiles().await {
                    tracing::error!("Failed to load profiles: {}", e);
                }
                if let Err(e) = state_clone.load_settings().await {
                    tracing::error!("Failed to load settings: {}", e);
                }
                match state_clone.restore_sessions().await {
                    Ok(sessions) if !sessions.is_empty() => {
                        tracing::info!("Restored {} session(s) for reconnection", sessions.len());
                    }
                    Err(e) => tracing::error!("Failed to restore sessions: {}", e),
                    _ => {}
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| {
            tracing::error!("Failed to run Tauri application: {}", e);
            e
        })
        .ok();
}
