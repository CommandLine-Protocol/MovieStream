use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::abstractions::{MediaRepository, MovieRepository};
use crate::error::AppError;
use crate::services::{PlaybackService, PlaybackSession};

#[tauri::command]
pub async fn start_playback(
    movie_id: String,
    media_id: String,
    playback_service: State<'_, Arc<PlaybackService>>,
    media_repo: State<'_, Arc<dyn MediaRepository>>,
    movie_repo: State<'_, Arc<dyn MovieRepository>>,
) -> Result<PlaybackSession, AppError> {
    let m_id = Uuid::parse_str(&movie_id).map_err(|e| AppError::Validation(e.to_string()))?;
    let med_id = Uuid::parse_str(&media_id).map_err(|e| AppError::Validation(e.to_string()))?;

    let media = media_repo
        .get_media(&med_id)?
        .ok_or_else(|| AppError::NotFound(format!("Media {} not found", media_id)))?;

    let movie = movie_repo
        .get_movie(&m_id)?
        .ok_or_else(|| AppError::NotFound(format!("Movie {} not found", movie_id)))?;

    playback_service.start_movie(m_id, med_id, &media.path, &movie.title)
}

#[tauri::command]
pub async fn play(playback_service: State<'_, Arc<PlaybackService>>) -> Result<(), AppError> {
    playback_service.play()
}

#[tauri::command]
pub async fn pause(playback_service: State<'_, Arc<PlaybackService>>) -> Result<(), AppError> {
    playback_service.pause()
}

#[tauri::command]
pub async fn stop(playback_service: State<'_, Arc<PlaybackService>>) -> Result<(), AppError> {
    playback_service.stop()
}

#[tauri::command]
pub async fn seek(
    position_seconds: u32,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.seek(position_seconds)
}

#[tauri::command]
pub async fn resume_at(
    position_seconds: u32,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.resume_from(position_seconds)
}

#[tauri::command]
pub async fn set_volume(
    level: u8,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.set_volume(level)
}

#[tauri::command]
pub async fn set_mute(
    muted: bool,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.set_mute(muted)
}

#[tauri::command]
pub async fn set_fullscreen(
    enabled: bool,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.set_fullscreen(enabled)
}

#[tauri::command]
pub async fn set_playback_speed(
    speed: f32,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.set_playback_speed(speed)
}

#[tauri::command]
pub async fn select_audio_track(
    track_id: String,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.select_audio_track(&track_id)
}

#[tauri::command]
pub async fn select_subtitle_track(
    track_id: Option<String>,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.select_subtitle_track(track_id.as_deref())
}

#[tauri::command]
pub async fn load_external_subtitle(
    path: String,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.load_external_subtitle(&path)
}

#[tauri::command]
pub async fn get_active_session(
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<Option<PlaybackSession>, AppError> {
    Ok(playback_service.get_active_session())
}

#[tauri::command]
pub async fn record_position(
    position_seconds: u32,
    playback_service: State<'_, Arc<PlaybackService>>,
) -> Result<(), AppError> {
    playback_service.record_position_tick(position_seconds)
}
