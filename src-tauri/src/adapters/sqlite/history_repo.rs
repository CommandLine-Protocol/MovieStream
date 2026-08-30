use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::abstractions::WatchHistoryRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{MetadataStatus, Movie, WatchHistoryEntry};
use crate::error::{AppError, AppResult};

pub struct SqliteWatchHistoryRepository {
    db: SqliteDb,
}

impl SqliteWatchHistoryRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_entry(row: &Row) -> Result<WatchHistoryEntry, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let movie_id_str: String = row.get(1)?;
        let started_at_str: String = row.get(2)?;
        let completed_at_str: Option<String> = row.get(3)?;
        let last_position_seconds: u32 = row.get(4)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let movie_id = Uuid::parse_str(&movie_id_str).unwrap_or_default();
        let started_at = DateTime::parse_from_rfc3339(&started_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let completed_at = completed_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Ok(WatchHistoryEntry {
            id,
            movie_id,
            started_at,
            completed_at,
            last_position_seconds,
        })
    }
}

impl WatchHistoryRepository for SqliteWatchHistoryRepository {
    fn add_entry(&self, entry: &WatchHistoryEntry) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let completed_at_str = entry.completed_at.map(|t| t.to_rfc3339());

        conn.execute(
            "
            INSERT INTO watch_history (id, movie_id, started_at, completed_at, last_position_seconds)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                entry.id.to_string(),
                entry.movie_id.to_string(),
                entry.started_at.to_rfc3339(),
                completed_at_str,
                entry.last_position_seconds,
            ],
        )?;

        Ok(())
    }

    fn update_entry(&self, entry: &WatchHistoryEntry) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let completed_at_str = entry.completed_at.map(|t| t.to_rfc3339());

        conn.execute(
            "
            UPDATE watch_history
            SET completed_at = ?1, last_position_seconds = ?2
            WHERE id = ?3
            ",
            params![
                completed_at_str,
                entry.last_position_seconds,
                entry.id.to_string(),
            ],
        )?;

        Ok(())
    }

    fn get_recent(&self, limit: usize) -> AppResult<Vec<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT m.id, m.title, m.original_title, m.year, m.description, m.poster_path, m.backdrop_path,
                   m.genres, m.[cast], m.director, m.rating, m.metadata_provider_id, m.metadata_status,
                   m.created_at, m.updated_at
            FROM movie m
            JOIN (
                SELECT movie_id, MAX(started_at) as max_started
                FROM watch_history
                GROUP BY movie_id
            ) recent ON recent.movie_id = m.id
            ORDER BY recent.max_started DESC
            LIMIT ?1
            ",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
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

    fn list_entries_for_movie(&self, movie_id: &Uuid) -> AppResult<Vec<WatchHistoryEntry>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT id, movie_id, started_at, completed_at, last_position_seconds
            FROM watch_history
            WHERE movie_id = ?1
            ORDER BY started_at DESC
            ",
        )?;

        let rows = stmt.query_map(params![movie_id.to_string()], Self::row_to_entry)?;
        let mut entries = Vec::new();
        for entry_res in rows {
            entries.push(entry_res?);
        }

        Ok(entries)
    }
}
