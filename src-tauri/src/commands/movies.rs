use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::domain::{Media, Movie, MovieFilter, MovieSort};
use crate::error::AppError;
use crate::services::{LibraryService, SearchService};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieWithMedia {
    pub movie: Movie,
    pub media: Vec<Media>,
}

#[tauri::command]
pub async fn list_movies(
    filter: Option<MovieFilter>,
    sort: Option<MovieSort>,
    library_service: State<'_, Arc<LibraryService>>,
) -> Result<Vec<Movie>, AppError> {
    let f = filter.unwrap_or_default();
    let s = sort.unwrap_or_default();
    library_service.list_movies(&f, s)
}

#[tauri::command]
pub async fn get_movie(
    movie_id: String,
    library_service: State<'_, Arc<LibraryService>>,
) -> Result<Option<MovieWithMedia>, AppError> {
    let id = Uuid::parse_str(&movie_id).map_err(|e| AppError::Validation(e.to_string()))?;
    match library_service.get_movie_with_media(&id)? {
        Some((movie, media)) => Ok(Some(MovieWithMedia { movie, media })),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn search_movies(
    query: String,
    search_service: State<'_, Arc<SearchService>>,
) -> Result<Vec<Movie>, AppError> {
    search_service.search(&query)
}

#[tauri::command]
pub async fn set_metadata_match(
    movie_id: String,
    provider_id: String,
    library_service: State<'_, Arc<LibraryService>>,
) -> Result<Movie, AppError> {
    let id = Uuid::parse_str(&movie_id).map_err(|e| AppError::Validation(e.to_string()))?;
    library_service.set_manual_metadata(&id, &provider_id)
}
