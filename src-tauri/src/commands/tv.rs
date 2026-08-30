use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::abstractions::{MediaRepository, TvRepository};
use crate::domain::{SeriesDetails, TvSeries};
use crate::error::AppError;
use crate::services::{PlaybackService, PlaybackSession};

#[tauri::command]
pub async fn list_tv_series(
    tv_repo: State<'_, Arc<dyn TvRepository>>,
) -> Result<Vec<TvSeries>, AppError> {
    tv_repo.list_series()
}

#[tauri::command]
pub async fn get_series_details(
    series_id: String,
    tv_repo: State<'_, Arc<dyn TvRepository>>,
) -> Result<Option<SeriesDetails>, AppError> {
    let s_id = Uuid::parse_str(&series_id).map_err(|e| AppError::Validation(e.to_string()))?;
    tv_repo.get_series_details(&s_id)
}

#[tauri::command]
pub async fn start_episode_playback(
    episode_id: String,
    media_id: String,
    playback_service: State<'_, Arc<PlaybackService>>,
    media_repo: State<'_, Arc<dyn MediaRepository>>,
    tv_repo: State<'_, Arc<dyn TvRepository>>,
) -> Result<PlaybackSession, AppError> {
    let ep_uuid = Uuid::parse_str(&episode_id).map_err(|e| AppError::Validation(e.to_string()))?;
    let med_uuid = Uuid::parse_str(&media_id).map_err(|e| AppError::Validation(e.to_string()))?;

    let media = media_repo
        .get_media(&med_uuid)?
        .ok_or_else(|| AppError::NotFound(format!("Media {} not found", media_id)))?;

    let episode = tv_repo
        .get_episode(&ep_uuid)?
        .ok_or_else(|| AppError::NotFound(format!("Episode {} not found", episode_id)))?;

    let series = tv_repo
        .get_series(&episode.series_id)?
        .ok_or_else(|| AppError::NotFound(format!("Series {} not found", episode.series_id)))?;

    playback_service.start_episode(
        ep_uuid,
        med_uuid,
        &media.path,
        series.id,
        episode.season_number,
        episode.episode_number,
        &series.title,
        &episode.title,
    )
}
