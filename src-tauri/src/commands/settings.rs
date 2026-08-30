use std::sync::Arc;
use tauri::State;

use crate::domain::AppSettings;
use crate::error::AppError;
use crate::services::SettingsService;

#[tauri::command]
pub async fn get_settings(
    settings_service: State<'_, Arc<SettingsService>>,
) -> Result<AppSettings, AppError> {
    settings_service.get()
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    settings_service: State<'_, Arc<SettingsService>>,
) -> Result<AppSettings, AppError> {
    settings_service.update(settings)
}
