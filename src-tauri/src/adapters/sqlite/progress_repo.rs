use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::abstractions::ProgressRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{ContinueWatchingItem, MediaProgress, MediaType};
use crate::error::{AppError, AppResult};

pub struct SqliteProgressRepository {
    db: SqliteDb,
}

impl SqliteProgressRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_progress(row: &Row) -> Result<MediaProgress, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let media_type_str: String = row.get(1)?;
        let media_id_str: String = row.get(2)?;
        let movie_id_str: Option<String> = row.get(3)?;
        let series_id_str: Option<String> = row.get(4)?;
        let season_number: Option<u32> = row.get(5)?;
        let episode_number: Option<u32> = row.get(6)?;
        let episode_id_str: Option<String> = row.get(7)?;
        let position_seconds: u32 = row.get(8)?;
        let duration_seconds: u32 = row.get(9)?;
        let progress_percentage: f64 = row.get(10)?;
        let completed_int: i32 = row.get(11)?;
        let last_watched_str: String = row.get(12)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let media_type = if media_type_str == "episode" {
            MediaType::Episode
        } else {
            MediaType::Movie
        };
        let media_id = Uuid::parse_str(&media_id_str).unwrap_or_default();
        let movie_id = movie_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let series_id = series_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let episode_id = episode_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let last_watched = DateTime::parse_from_rfc3339(&last_watched_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(MediaProgress {
            id,
            media_type,
            media_id,
            movie_id,
            series_id,
            season_number,
            episode_number,
            episode_id,
            position_seconds,
            duration_seconds,
            progress_percentage: progress_percentage as f32,
            completed: completed_int == 1,
            last_watched,
        })
    }
}

impl ProgressRepository for SqliteProgressRepository {
    fn save_progress(&self, progress: &MediaProgress) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let percentage = if progress.duration_seconds > 0 {
            (progress.position_seconds as f32 / progress.duration_seconds as f32) * 100.0
        } else {
            0.0
        };

        // Determine unique progress record by media_id
        conn.execute(
            "
            INSERT INTO playback_progress (
                id, media_type, media_id, movie_id, series_id, season_number,
                episode_number, episode_id, position_seconds, duration_seconds,
                progress_percentage, completed, last_watched
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                position_seconds = excluded.position_seconds,
                duration_seconds = excluded.duration_seconds,
                progress_percentage = excluded.progress_percentage,
                completed = excluded.completed,
                last_watched = excluded.last_watched
            ",
            params![
                progress.id.to_string(),
                progress.media_type.to_string(),
                progress.media_id.to_string(),
                progress.movie_id.map(|u| u.to_string()),
                progress.series_id.map(|u| u.to_string()),
                progress.season_number,
                progress.episode_number,
                progress.episode_id.map(|u| u.to_string()),
                progress.position_seconds,
                progress.duration_seconds,
                percentage as f64,
                if progress.completed { 1 } else { 0 },
                progress.last_watched.to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        // Sync legacy movie playback_state if movie
        if let Some(m_id) = progress.movie_id {
            let _ = conn.execute(
                "
                INSERT INTO playback_state (movie_id, media_id, position_seconds, duration_seconds, completed, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(movie_id) DO UPDATE SET
                    media_id = excluded.media_id,
                    position_seconds = excluded.position_seconds,
                    duration_seconds = excluded.duration_seconds,
                    completed = excluded.completed,
                    updated_at = excluded.updated_at
                ",
                params![
                    m_id.to_string(),
                    progress.media_id.to_string(),
                    progress.position_seconds,
                    progress.duration_seconds,
                    if progress.completed { 1 } else { 0 },
                    progress.last_watched.to_rfc3339(),
                ],
            );
        }

        Ok(())
    }

    fn get_progress_by_media(&self, media_id: &Uuid) -> AppResult<Option<MediaProgress>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "
            SELECT id, media_type, media_id, movie_id, series_id, season_number,
                   episode_number, episode_id, position_seconds, duration_seconds,
                   progress_percentage, completed, last_watched
            FROM playback_progress
            WHERE media_id = ?1
            LIMIT 1
            "
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![media_id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_progress(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn get_progress_by_movie(&self, movie_id: &Uuid) -> AppResult<Option<MediaProgress>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "
            SELECT id, media_type, media_id, movie_id, series_id, season_number,
                   episode_number, episode_id, position_seconds, duration_seconds,
                   progress_percentage, completed, last_watched
            FROM playback_progress
            WHERE movie_id = ?1
            ORDER BY last_watched DESC
            LIMIT 1
            "
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![movie_id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_progress(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn get_progress_by_episode(&self, episode_id: &Uuid) -> AppResult<Option<MediaProgress>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "
            SELECT id, media_type, media_id, movie_id, series_id, season_number,
                   episode_number, episode_id, position_seconds, duration_seconds,
                   progress_percentage, completed, last_watched
            FROM playback_progress
            WHERE episode_id = ?1
            ORDER BY last_watched DESC
            LIMIT 1
            "
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![episode_id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_progress(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn mark_completed(&self, media_id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "
            UPDATE playback_progress
            SET completed = 1, position_seconds = 0, progress_percentage = 100.0, last_watched = datetime('now')
            WHERE media_id = ?1
            ",
            params![media_id.to_string()],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    fn get_continue_watching(&self, limit: usize) -> AppResult<Vec<ContinueWatchingItem>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT p.id, p.media_type, p.media_id, p.movie_id, p.series_id, p.season_number,
                   p.episode_number, p.episode_id, p.position_seconds, p.duration_seconds,
                   p.progress_percentage, p.completed, p.last_watched,
                   m.title, m.poster_path, m.backdrop_path, m.year,
                   s.title, s.poster_path,
                   e.title, e.still_path
            FROM playback_progress p
            LEFT JOIN movie m ON m.id = p.movie_id
            LEFT JOIN tv_series s ON s.id = p.series_id
            LEFT JOIN tv_episode e ON e.id = p.episode_id
            WHERE p.completed = 0 AND p.position_seconds > 10
            ORDER BY p.last_watched DESC
            LIMIT ?1
            "
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let progress = Self::row_to_progress(row)?;
            let movie_title: Option<String> = row.get(13)?;
            let movie_poster: Option<String> = row.get(14)?;
            let movie_backdrop: Option<String> = row.get(15)?;
            let movie_year: Option<u16> = row.get(16)?;
            let series_title: Option<String> = row.get(17)?;
            let series_poster: Option<String> = row.get(18)?;
            let episode_title: Option<String> = row.get(19)?;
            let episode_still: Option<String> = row.get(20)?;

            Ok(ContinueWatchingItem {
                progress,
                movie_title,
                movie_poster,
                movie_backdrop,
                movie_year,
                series_title,
                series_poster,
                episode_title,
                episode_still,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(items)
    }

    fn delete_progress(&self, media_id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM playback_progress WHERE media_id = ?1", params![media_id.to_string()])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
