use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::abstractions::MediaRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{AudioTrackInfo, Media, MediaAvailability, SubtitleTrackInfo};
use crate::error::{AppError, AppResult};

pub struct SqliteMediaRepository {
    db: SqliteDb,
}

impl SqliteMediaRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_media(row: &Row) -> Result<Media, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let movie_id_str: Option<String> = row.get(1)?;
        let episode_id_str: Option<String> = row.get(2)?;
        let source_id_str: String = row.get(3)?;
        let path: String = row.get(4)?;
        let size_bytes: i64 = row.get(5)?;
        let duration_seconds: Option<u32> = row.get(6)?;
        let container_format: Option<String> = row.get(7)?;
        let video_codec: Option<String> = row.get(8)?;
        let resolution_width: Option<u32> = row.get(9)?;
        let resolution_height: Option<u32> = row.get(10)?;
        let audio_tracks_raw: Option<String> = row.get(11)?;
        let subtitle_tracks_raw: Option<String> = row.get(12)?;
        let file_hash: Option<String> = row.get(13)?;
        let file_mtime_str: String = row.get(14)?;
        let availability_str: String = row.get(15)?;
        let created_at_str: String = row.get(16)?;
        let updated_at_str: String = row.get(17)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let movie_id = movie_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let episode_id = episode_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let source_id = Uuid::parse_str(&source_id_str).unwrap_or_default();
        let audio_tracks: Vec<AudioTrackInfo> = audio_tracks_raw
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let subtitle_tracks: Vec<SubtitleTrackInfo> = subtitle_tracks_raw
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let availability = MediaAvailability::from_str(&availability_str).unwrap_or_default();
        let file_mtime = DateTime::parse_from_rfc3339(&file_mtime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Media {
            id,
            movie_id,
            episode_id,
            source_id,
            path,
            size_bytes: size_bytes as u64,
            duration_seconds,
            container_format,
            video_codec,
            resolution_width,
            resolution_height,
            audio_tracks,
            subtitle_tracks,
            file_hash,
            file_mtime,
            availability,
            created_at,
            updated_at,
        })
    }
}

const SELECT_FIELDS: &str = "id, movie_id, episode_id, source_id, path, size_bytes, duration_seconds, container_format, video_codec, resolution_width, resolution_height, audio_tracks, subtitle_tracks, file_hash, file_mtime, availability, created_at, updated_at";

impl MediaRepository for SqliteMediaRepository {
    fn upsert_media(&self, media: &Media) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let audio_json = serde_json::to_string(&media.audio_tracks).unwrap_or_else(|_| "[]".to_string());
        let subtitle_json = serde_json::to_string(&media.subtitle_tracks).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "
            INSERT INTO media (
                id, movie_id, episode_id, source_id, path, size_bytes, duration_seconds,
                container_format, video_codec, resolution_width, resolution_height,
                audio_tracks, subtitle_tracks, file_hash, file_mtime, availability,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            ON CONFLICT(path) DO UPDATE SET
                movie_id = excluded.movie_id,
                episode_id = excluded.episode_id,
                source_id = excluded.source_id,
                size_bytes = excluded.size_bytes,
                duration_seconds = excluded.duration_seconds,
                container_format = excluded.container_format,
                video_codec = excluded.video_codec,
                resolution_width = excluded.resolution_width,
                resolution_height = excluded.resolution_height,
                audio_tracks = excluded.audio_tracks,
                subtitle_tracks = excluded.subtitle_tracks,
                file_hash = excluded.file_hash,
                file_mtime = excluded.file_mtime,
                availability = excluded.availability,
                updated_at = excluded.updated_at
            ",
            params![
                media.id.to_string(),
                media.movie_id.map(|u| u.to_string()),
                media.episode_id.map(|u| u.to_string()),
                media.source_id.to_string(),
                media.path,
                media.size_bytes as i64,
                media.duration_seconds,
                media.container_format,
                media.video_codec,
                media.resolution_width,
                media.resolution_height,
                audio_json,
                subtitle_json,
                media.file_hash,
                media.file_mtime.to_rfc3339(),
                media.availability.to_string(),
                media.created_at.to_rfc3339(),
                media.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_media(&self, id: &Uuid) -> AppResult<Option<Media>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let query = format!("SELECT {} FROM media WHERE id = ?1", SELECT_FIELDS);
        let mut stmt = conn.prepare(&query).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_media(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn find_by_path(&self, path: &str) -> AppResult<Option<Media>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let query = format!("SELECT {} FROM media WHERE path = ?1", SELECT_FIELDS);
        let mut stmt = conn.prepare(&query).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![path]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_media(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn list_media_for_movie(&self, movie_id: &Uuid) -> AppResult<Vec<Media>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let query = format!("SELECT {} FROM media WHERE movie_id = ?1 ORDER BY COALESCE(resolution_height, 0) DESC, created_at ASC", SELECT_FIELDS);
        let mut stmt = conn.prepare(&query).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![movie_id.to_string()], |row| Self::row_to_media(row))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    fn list_media_for_episode(&self, episode_id: &Uuid) -> AppResult<Vec<Media>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let query = format!("SELECT {} FROM media WHERE episode_id = ?1 ORDER BY created_at ASC", SELECT_FIELDS);
        let mut stmt = conn.prepare(&query).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![episode_id.to_string()], |row| Self::row_to_media(row))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    fn list_media_for_source(&self, source_id: &Uuid) -> AppResult<Vec<Media>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let query = format!("SELECT {} FROM media WHERE source_id = ?1", SELECT_FIELDS);
        let mut stmt = conn.prepare(&query).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![source_id.to_string()], |row| Self::row_to_media(row))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    fn set_availability(&self, media_id: &Uuid, availability: MediaAvailability) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "UPDATE media SET availability = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![availability.to_string(), media_id.to_string()],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    fn set_source_media_availability(&self, source_id: &Uuid, availability: MediaAvailability) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "UPDATE media SET availability = ?1, updated_at = datetime('now') WHERE source_id = ?2",
            params![availability.to_string(), source_id.to_string()],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    fn delete_media(&self, id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM media WHERE id = ?1", params![id.to_string()])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
