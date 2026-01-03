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
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting RUSSH Client");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::new())
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
