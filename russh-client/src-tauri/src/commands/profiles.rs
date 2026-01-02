//! Profile management Tauri commands

use tauri::State;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::{AppState, ProfileData};

/// Create a new connection profile
#[tauri::command]
pub async fn profile_create(
    state: State<'_, AppState>,
    profile: ProfileData,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    tracing::info!("Creating profile: {} ({})", profile.name, id);
    
    state.save_profile(id.clone(), profile).await?;
    Ok(id)
}

/// Update an existing connection profile
#[tauri::command]
pub async fn profile_update(
    state: State<'_, AppState>,
    profile: ProfileData,
) -> Result<(), AppError> {
    let id = profile.id.clone()
        .ok_or(AppError::MissingProfileId)?;
    
    tracing::info!("Updating profile: {} ({})", profile.name, id);
    state.update_profile(id, profile).await
}

/// Delete a connection profile
#[tauri::command]
pub async fn profile_delete(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), AppError> {
    tracing::info!("Deleting profile: {}", profile_id);
    state.delete_profile(&profile_id).await
}

/// List all connection profiles
#[tauri::command]
pub async fn profile_list(
    state: State<'_, AppState>,
) -> Result<Vec<ProfileData>, AppError> {
    // Load profiles from disk on first call
    state.load_profiles().await.ok();
    Ok(state.list_profiles().await)
}

/// Export profiles to JSON
#[tauri::command]
pub async fn profile_export(
    state: State<'_, AppState>,
    include_credentials: bool,
) -> Result<String, AppError> {
    tracing::info!("Exporting profiles (include_credentials: {})", include_credentials);
    state.export_profiles(include_credentials).await
}

/// Import profiles from JSON
#[tauri::command]
pub async fn profile_import(
    state: State<'_, AppState>,
    json_data: String,
) -> Result<usize, AppError> {
    tracing::info!("Importing profiles");
    state.import_profiles(&json_data).await
}
