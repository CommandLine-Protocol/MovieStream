use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use crate::domain::{Movie, PlaybackState};
use crate::error::AppError;
use crate::services::HistoryService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinueWatchingItem {
    pub movie: Movie,
    pub state: PlaybackState,
}

#[tauri::command]
pub async fn recently_watched(
    limit: Option<usize>,
    history_service: State<'_, Arc<HistoryService>>,
) -> Result<Vec<Movie>, AppError> {
    let l = limit.unwrap_or(20);
    history_service.recently_watched(l)
}

#[tauri::command]
pub async fn continue_watching(
    history_service: State<'_, Arc<HistoryService>>,
) -> Result<Vec<ContinueWatchingItem>, AppError> {
    let items = history_service.continue_watching()?;
    let result = items
        .into_iter()
        .map(|(movie, state)| ContinueWatchingItem { movie, state })
        .collect();
    Ok(result)
}
