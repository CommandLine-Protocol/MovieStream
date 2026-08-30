use std::sync::Arc;
use tauri::State;

use crate::error::AppError;
use crate::services::tmdb_service::{
    TmdbEpisodeDetail, TmdbMovieDetail, TmdbMovieResult, TmdbSeasonSummary, TmdbService,
    TmdbTrendingItem, TmdbTvDetail, TmdbTvResult,
};

#[tauri::command]
pub async fn search_tmdb_movies(
    query: String,
    year: Option<u16>,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbMovieResult>, AppError> {
    tmdb_service.search_movies(&query, year).await
}

#[tauri::command]
pub async fn get_movie_details(
    movie_id: i64,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<TmdbMovieDetail, AppError> {
    tmdb_service.get_movie_details(movie_id).await
}

#[tauri::command]
pub async fn search_tv(
    query: String,
    year: Option<u16>,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbTvResult>, AppError> {
    tmdb_service.search_tv(&query, year).await
}

#[tauri::command]
pub async fn get_tv_details(
    series_id: i64,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<TmdbTvDetail, AppError> {
    tmdb_service.get_tv_details(series_id).await
}

#[tauri::command]
pub async fn get_tv_seasons(
    series_id: i64,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbSeasonSummary>, AppError> {
    let details = tmdb_service.get_tv_details(series_id).await?;
    Ok(details.seasons)
}

#[tauri::command]
pub async fn get_tv_episodes(
    series_id: i64,
    season_number: u32,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbEpisodeDetail>, AppError> {
    tmdb_service.get_tv_episodes(series_id, season_number).await
}

#[tauri::command]
pub async fn get_trending(
    media_type: Option<String>,
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbTrendingItem>, AppError> {
    let m = media_type.unwrap_or_else(|| "movie".to_string());
    tmdb_service.get_trending(&m).await
}

#[tauri::command]
pub async fn get_popular_movies(
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbMovieResult>, AppError> {
    tmdb_service.search_movies("popular", None).await
}

#[tauri::command]
pub async fn get_popular_tv(
    tmdb_service: State<'_, Arc<TmdbService>>,
) -> Result<Vec<TmdbTvResult>, AppError> {
    tmdb_service.search_tv("popular", None).await
}
