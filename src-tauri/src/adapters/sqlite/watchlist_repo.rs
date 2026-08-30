use chrono::{DateTime, Utc};
use rusqlite::params;
use std::str::FromStr;
use uuid::Uuid;

use crate::abstractions::WatchlistRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{MetadataStatus, Movie};
use crate::error::{AppError, AppResult};

pub struct SqliteWatchlistRepository {
    db: SqliteDb,
}

impl SqliteWatchlistRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }
}

impl WatchlistRepository for SqliteWatchlistRepository {
    fn add_to_watchlist(&self, movie_id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "
            INSERT INTO watchlist (movie_id, added_at)
            VALUES (?1, ?2)
            ON CONFLICT(movie_id) DO NOTHING
            ",
            params![movie_id.to_string(), Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    fn remove_from_watchlist(&self, movie_id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "DELETE FROM watchlist WHERE movie_id = ?1",
            params![movie_id.to_string()],
        )?;

        Ok(())
    }

    fn is_in_watchlist(&self, movie_id: &Uuid) -> AppResult<bool> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT 1 FROM watchlist WHERE movie_id = ?1 LIMIT 1")?;
        let exists = stmt.exists(params![movie_id.to_string()])?;
        Ok(exists)
    }

    fn list_watchlist(&self) -> AppResult<Vec<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT m.id, m.title, m.original_title, m.year, m.description, m.poster_path, m.backdrop_path,
                   m.genres, m.[cast], m.director, m.rating, m.metadata_provider_id, m.metadata_status,
                   m.created_at, m.updated_at
            FROM watchlist w
            JOIN movie m ON m.id = w.movie_id
            ORDER BY w.added_at DESC
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let original_title: Option<String> = row.get(2)?;
            let year: Option<u16> = row.get(3)?;
            let description: Option<String> = row.get(4)?;
            let poster_path: Option<String> = row.get(5)?;
            let backdrop_path: Option<String> = row.get(6)?;
            let genres_raw: Option<String> = row.get(7)?;
            let cast_raw: Option<String> = row.get(8)?;
            let director: Option<String> = row.get(9)?;
            let rating: Option<f32> = row.get(10)?;
            let metadata_provider_id: Option<String> = row.get(11)?;
            let metadata_status_str: String = row.get(12)?;
            let created_at_str: String = row.get(13)?;
            let updated_at_str: String = row.get(14)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let metadata_status = MetadataStatus::from_str(&metadata_status_str).unwrap_or_default();
            let genres: Vec<String> = genres_raw
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let cast: Vec<String> = cast_raw
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Movie {
                id,
                title,
                original_title,
                year,
                description,
                poster_path,
                backdrop_path,
                genres,
                cast,
                director,
                rating,
                metadata_provider_id,
                metadata_status,
                created_at,
                updated_at,
            })
        })?;

        let mut movies = Vec::new();
        for movie_res in rows {
            movies.push(movie_res?);
        }

        Ok(movies)
    }
}
