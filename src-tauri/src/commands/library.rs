use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::domain::LibrarySource;
use crate::error::AppError;
use crate::events::{EVENT_SCAN_PROGRESS, ScanProgressPayload};
use crate::services::{LibraryService, MediaStreamServer};

#[tauri::command]
pub async fn add_source(
    path: String,
    service: State<'_, Arc<LibraryService>>,
    stream_server: State<'_, Arc<MediaStreamServer>>,
    app: AppHandle,
) -> Result<LibrarySource, AppError> {
    // Register this directory as allowed for streaming
    stream_server.register_allowed_path(std::path::Path::new(&path));

    let app_handle = app.clone();
    let emitter: Arc<dyn Fn(ScanProgressPayload) + Send + Sync> = Arc::new(move |payload| {
        let _ = app_handle.emit(EVENT_SCAN_PROGRESS, payload);
    });

    service.add_source(&path, Some(emitter)).await
}

#[tauri::command]
pub async fn pick_and_add_source(
    service: State<'_, Arc<LibraryService>>,
    stream_server: State<'_, Arc<MediaStreamServer>>,
    app: AppHandle,
) -> Result<Option<LibrarySource>, AppError> {
    let app_handle = app.clone();
    let emitter: Arc<dyn Fn(ScanProgressPayload) + Send + Sync> = Arc::new(move |payload| {
        let _ = app_handle.emit(EVENT_SCAN_PROGRESS, payload);
    });

    let result = service.pick_and_add_source(Some(emitter)).await?;
    if let Some(ref source) = result {
        stream_server.register_allowed_path(std::path::Path::new(&source.path));
    }
    Ok(result)
}

#[tauri::command]
pub async fn remove_source(
    source_id: String,
    service: State<'_, Arc<LibraryService>>,
    stream_server: State<'_, Arc<MediaStreamServer>>,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(&source_id).map_err(|e| AppError::Validation(e.to_string()))?;

    // Unregister the source path before removing
    if let Ok(sources) = service.list_sources() {
        if let Some(source) = sources.iter().find(|s| s.id == id) {
            stream_server.unregister_allowed_path(std::path::Path::new(&source.path));
        }
    }

    service.remove_source(&id)
}

#[tauri::command]
pub async fn list_sources(
    service: State<'_, Arc<LibraryService>>,
) -> Result<Vec<LibrarySource>, AppError> {
    service.list_sources()
}

#[tauri::command]
pub async fn rescan_source(
    source_id: String,
    service: State<'_, Arc<LibraryService>>,
    app: AppHandle,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(&source_id).map_err(|e| AppError::Validation(e.to_string()))?;
    let app_handle = app.clone();
    let emitter: Arc<dyn Fn(ScanProgressPayload) + Send + Sync> = Arc::new(move |payload| {
        let _ = app_handle.emit(EVENT_SCAN_PROGRESS, payload);
    });

    service.rescan_source(&id, Some(emitter)).await
}

#[tauri::command]
pub async fn rescan_all(
    service: State<'_, Arc<LibraryService>>,
    app: AppHandle,
) -> Result<(), AppError> {
    let app_handle = app.clone();
    let emitter: Arc<dyn Fn(ScanProgressPayload) + Send + Sync> = Arc::new(move |payload| {
        let _ = app_handle.emit(EVENT_SCAN_PROGRESS, payload);
    });

    service.rescan_all(Some(emitter)).await
}
