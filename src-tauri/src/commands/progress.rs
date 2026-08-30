use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::abstractions::ProgressRepository;
use crate::domain::{ContinueWatchingItem, MediaProgress, TvEpisode};
use crate::error::AppError;
use crate::services::PlaybackService;

#[tauri::command]
pub async fn get_continue_watching(
    limit: Option<usize>,
    progress_repo: State<'_, Arc<dyn ProgressRepository>>,
) -> Result<Vec<ContinueWatchingItem>, AppError> {
    progress_repo.get_continue_watching(limit.unwrap_or(20))
}

#[tauri::command]
pub async fn get_playback_progress(
    media_id: String,
    progress_repo: State<'_, Arc<dyn ProgressRepository>>,
) -> Result<Option<MediaProgress>, AppError> {
    let m_id = Uuid::parse_str(&media_id).map_err(|e| AppError::Validation(e.to_string()))?;
    progress_repo.get_progress_by_media(&m_id)
}

#[tauri::command]
pub async fn mark_media_completed(
    media_id: String,
    progress_repo: State<'_, Arc<dyn ProgressRepository>>,
) -> Result<(), AppError> {
    let m_id = Uuid::parse_str(&media_id).map_err(|e| AppError::Validation(e.to_string()))?;
    progress_repo.mark_completed(&m_id)
}

#[tauri::command]
pub async fn get_next_episode(
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<Option<TvEpisode>, AppError> {
    playback_service.get_next_episode()
}
