use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::domain::Movie;
use crate::error::AppError;
use crate::services::WatchlistService;

#[tauri::command]
pub async fn add_to_watchlist(
    movie_id: String,
    watchlist_service: State<'_, Arc<WatchlistService>>,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(&movie_id).map_err(|e| AppError::Validation(e.to_string()))?;
    watchlist_service.add(&id)
}

#[tauri::command]
pub async fn remove_from_watchlist(
    movie_id: String,
    watchlist_service: State<'_, Arc<WatchlistService>>,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(&movie_id).map_err(|e| AppError::Validation(e.to_string()))?;
    watchlist_service.remove(&id)
}

#[tauri::command]
pub async fn is_in_watchlist(
    movie_id: String,
    watchlist_service: State<'_, Arc<WatchlistService>>,
) -> Result<bool, AppError> {
    let id = Uuid::parse_str(&movie_id).map_err(|e| AppError::Validation(e.to_string()))?;
    watchlist_service.is_in_watchlist(&id)
}

#[tauri::command]
pub async fn list_watchlist(
    watchlist_service: State<'_, Arc<WatchlistService>>,
) -> Result<Vec<Movie>, AppError> {
    watchlist_service.list()
}
