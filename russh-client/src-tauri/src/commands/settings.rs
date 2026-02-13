//! Settings management Tauri commands

use tauri::State;

use crate::error::AppError;
use crate::state::{AppSettings, AppState};

/// Load application settings
#[tauri::command]
pub async fn settings_load(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    tracing::info!("Loading settings");
    state.load_settings().await
}

/// Save application settings
#[tauri::command]
pub async fn settings_save(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), AppError> {
    tracing::info!("Saving settings");
    state.save_settings(settings).await
}
