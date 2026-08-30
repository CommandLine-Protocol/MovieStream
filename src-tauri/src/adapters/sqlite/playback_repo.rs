use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::abstractions::PlaybackStateRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{MetadataStatus, Movie, PlaybackState};
use crate::error::{AppError, AppResult};

pub struct SqlitePlaybackStateRepository {
    db: SqliteDb,
}

impl SqlitePlaybackStateRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_state(row: &Row) -> Result<PlaybackState, rusqlite::Error> {
        let movie_id_str: String = row.get(0)?;
        let media_id_str: String = row.get(1)?;
        let position_seconds: u32 = row.get(2)?;
        let duration_seconds: u32 = row.get(3)?;
        let completed_int: i32 = row.get(4)?;
        let updated_at_str: String = row.get(5)?;

        let movie_id = Uuid::parse_str(&movie_id_str).unwrap_or_default();
        let media_id = Uuid::parse_str(&media_id_str).unwrap_or_default();
        let completed = completed_int == 1;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(PlaybackState {
            movie_id,
            media_id,
            position_seconds,
            duration_seconds,
            completed,
            updated_at,
        })
    }
}

impl PlaybackStateRepository for SqlitePlaybackStateRepository {
    fn upsert_state(&self, state: &PlaybackState) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "
            INSERT INTO playback_state (
                movie_id, media_id, position_seconds, duration_seconds, completed, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(movie_id) DO UPDATE SET
                media_id = excluded.media_id,
                position_seconds = excluded.position_seconds,
                duration_seconds = excluded.duration_seconds,
                completed = excluded.completed,
                updated_at = excluded.updated_at
            ",
            params![
                state.movie_id.to_string(),
                state.media_id.to_string(),
                state.position_seconds,
                state.duration_seconds,
                if state.completed { 1 } else { 0 },
                state.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    fn get_state(&self, movie_id: &Uuid) -> AppResult<Option<PlaybackState>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT movie_id, media_id, position_seconds, duration_seconds, completed, updated_at
            FROM playback_state
            WHERE movie_id = ?1
            ",
        )?;

        let mut rows = stmt.query(params![movie_id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_state(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_in_progress(&self) -> AppResult<Vec<(Movie, PlaybackState)>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT m.id, m.title, m.original_title, m.year, m.description, m.poster_path, m.backdrop_path,
                   m.genres, m.[cast], m.director, m.rating, m.metadata_provider_id, m.metadata_status,
                   m.created_at, m.updated_at,
                   ps.movie_id, ps.media_id, ps.position_seconds, ps.duration_seconds, ps.completed, ps.updated_at
            FROM playback_state ps
            JOIN movie m ON m.id = ps.movie_id
            WHERE ps.completed = 0 AND ps.position_seconds > 0
            ORDER BY ps.updated_at DESC
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
            let m_created_at_str: String = row.get(13)?;
            let m_updated_at_str: String = row.get(14)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let metadata_status = MetadataStatus::from_str(&metadata_status_str).unwrap_or_default();
            let genres: Vec<String> = genres_raw
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let cast: Vec<String> = cast_raw
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let m_created_at = DateTime::parse_from_rfc3339(&m_created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let m_updated_at = DateTime::parse_from_rfc3339(&m_updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let movie = Movie {
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
                created_at: m_created_at,
                updated_at: m_updated_at,
            };

            let ps_movie_id_str: String = row.get(15)?;
            let ps_media_id_str: String = row.get(16)?;
            let position_seconds: u32 = row.get(17)?;
            let duration_seconds: u32 = row.get(18)?;
            let completed_int: i32 = row.get(19)?;
            let ps_updated_at_str: String = row.get(20)?;

            let ps_movie_id = Uuid::parse_str(&ps_movie_id_str).unwrap_or_default();
            let ps_media_id = Uuid::parse_str(&ps_media_id_str).unwrap_or_default();
            let ps_updated_at = DateTime::parse_from_rfc3339(&ps_updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let state = PlaybackState {
                movie_id: ps_movie_id,
                media_id: ps_media_id,
                position_seconds,
                duration_seconds,
                completed: completed_int == 1,
                updated_at: ps_updated_at,
            };

            Ok((movie, state))
        })?;

        let mut results = Vec::new();
        for res in rows {
            results.push(res?);
        }

        Ok(results)
    }

    fn delete_state(&self, movie_id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "DELETE FROM playback_state WHERE movie_id = ?1",
            params![movie_id.to_string()],
        )?;
        Ok(())
    }
}
