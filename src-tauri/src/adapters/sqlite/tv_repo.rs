use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::abstractions::TvRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{
    EpisodeWithMedia, MetadataStatus, SeasonWithEpisodes, SeriesDetails, TvEpisode, TvSeason,
    TvSeries,
};
use crate::error::{AppError, AppResult};

pub struct SqliteTvRepository {
    db: SqliteDb,
}

impl SqliteTvRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_series(row: &Row) -> Result<TvSeries, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let tmdb_id: Option<i64> = row.get(1)?;
        let title: String = row.get(2)?;
        let original_title: Option<String> = row.get(3)?;
        let year: Option<u16> = row.get(4)?;
        let description: Option<String> = row.get(5)?;
        let poster_path: Option<String> = row.get(6)?;
        let backdrop_path: Option<String> = row.get(7)?;
        let genres_raw: Option<String> = row.get(8)?;
        let rating: Option<f32> = row.get(9)?;
        let metadata_provider_id: Option<String> = row.get(10)?;
        let metadata_status_str: String = row.get(11)?;
        let created_at_str: String = row.get(12)?;
        let updated_at_str: String = row.get(13)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let genres: Vec<String> = genres_raw
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let metadata_status = match metadata_status_str.as_str() {
            "auto_matched" => MetadataStatus::AutoMatched,
            "manually_matched" => MetadataStatus::ManuallyMatched,
            _ => MetadataStatus::Unmatched,
        };
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(TvSeries {
            id,
            tmdb_id,
            title,
            original_title,
            year,
            description,
            poster_path,
            backdrop_path,
            genres,
            rating,
            metadata_provider_id,
            metadata_status,
            created_at,
            updated_at,
        })
    }

    fn row_to_season(row: &Row) -> Result<TvSeason, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let series_id_str: String = row.get(1)?;
        let season_number: u32 = row.get(2)?;
        let name: String = row.get(3)?;
        let overview: Option<String> = row.get(4)?;
        let poster_path: Option<String> = row.get(5)?;
        let episode_count: u32 = row.get(6)?;
        let created_at_str: String = row.get(7)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let series_id = Uuid::parse_str(&series_id_str).unwrap_or_default();
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(TvSeason {
            id,
            series_id,
            season_number,
            name,
            overview,
            poster_path,
            episode_count,
            created_at,
        })
    }

    fn row_to_episode(row: &Row) -> Result<TvEpisode, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let series_id_str: String = row.get(1)?;
        let season_id_str: String = row.get(2)?;
        let season_number: u32 = row.get(3)?;
        let episode_number: u32 = row.get(4)?;
        let title: String = row.get(5)?;
        let overview: Option<String> = row.get(6)?;
        let still_path: Option<String> = row.get(7)?;
        let air_date: Option<String> = row.get(8)?;
        let duration_seconds: Option<u32> = row.get(9)?;
        let rating: Option<f32> = row.get(10)?;
        let created_at_str: String = row.get(11)?;
        let updated_at_str: String = row.get(12)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let series_id = Uuid::parse_str(&series_id_str).unwrap_or_default();
        let season_id = Uuid::parse_str(&season_id_str).unwrap_or_default();
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(TvEpisode {
            id,
            series_id,
            season_id,
            season_number,
            episode_number,
            title,
            overview,
            still_path,
            air_date,
            duration_seconds,
            rating,
            created_at,
            updated_at,
        })
    }
}

impl TvRepository for SqliteTvRepository {
    fn upsert_series(&self, series: &TvSeries) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let genres_json = serde_json::to_string(&series.genres).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "
            INSERT INTO tv_series (
                id, tmdb_id, title, original_title, year, description,
                poster_path, backdrop_path, genres, rating, metadata_provider_id,
                metadata_status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(id) DO UPDATE SET
                tmdb_id = excluded.tmdb_id,
                title = excluded.title,
                original_title = excluded.original_title,
                year = excluded.year,
                description = excluded.description,
                poster_path = excluded.poster_path,
                backdrop_path = excluded.backdrop_path,
                genres = excluded.genres,
                rating = excluded.rating,
                metadata_provider_id = excluded.metadata_provider_id,
                metadata_status = excluded.metadata_status,
                updated_at = excluded.updated_at
            ",
            params![
                series.id.to_string(),
                series.tmdb_id,
                series.title,
                series.original_title,
                series.year,
                series.description,
                series.poster_path,
                series.backdrop_path,
                genres_json,
                series.rating,
                series.metadata_provider_id,
                series.metadata_status.to_string(),
                series.created_at.to_rfc3339(),
                series.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_series(&self, id: &Uuid) -> AppResult<Option<TvSeries>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, tmdb_id, title, original_title, year, description, poster_path, backdrop_path, genres, rating, metadata_provider_id, metadata_status, created_at, updated_at FROM tv_series WHERE id = ?1"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_series(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn list_series(&self) -> AppResult<Vec<TvSeries>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, tmdb_id, title, original_title, year, description, poster_path, backdrop_path, genres, rating, metadata_provider_id, metadata_status, created_at, updated_at FROM tv_series ORDER BY title ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map([], |row| Self::row_to_series(row))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    fn find_series_by_title(&self, title: &str) -> AppResult<Option<TvSeries>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, tmdb_id, title, original_title, year, description, poster_path, backdrop_path, genres, rating, metadata_provider_id, metadata_status, created_at, updated_at FROM tv_series WHERE LOWER(title) = LOWER(?1) LIMIT 1"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![title]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_series(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn delete_series(&self, id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM tv_series WHERE id = ?1", params![id.to_string()])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    fn upsert_season(&self, season: &TvSeason) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "
            INSERT INTO tv_season (
                id, series_id, season_number, name, overview, poster_path, episode_count, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                overview = excluded.overview,
                poster_path = excluded.poster_path,
                episode_count = excluded.episode_count
            ",
            params![
                season.id.to_string(),
                season.series_id.to_string(),
                season.season_number,
                season.name,
                season.overview,
                season.poster_path,
                season.episode_count,
                season.created_at.to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_season(&self, id: &Uuid) -> AppResult<Option<TvSeason>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, series_id, season_number, name, overview, poster_path, episode_count, created_at FROM tv_season WHERE id = ?1"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_season(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn list_seasons_by_series(&self, series_id: &Uuid) -> AppResult<Vec<TvSeason>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, series_id, season_number, name, overview, poster_path, episode_count, created_at FROM tv_season WHERE series_id = ?1 ORDER BY season_number ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![series_id.to_string()], |row| Self::row_to_season(row))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    fn upsert_episode(&self, episode: &TvEpisode) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "
            INSERT INTO tv_episode (
                id, series_id, season_id, season_number, episode_number, title, overview,
                still_path, air_date, duration_seconds, rating, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                overview = excluded.overview,
                still_path = excluded.still_path,
                air_date = excluded.air_date,
                duration_seconds = excluded.duration_seconds,
                rating = excluded.rating,
                updated_at = excluded.updated_at
            ",
            params![
                episode.id.to_string(),
                episode.series_id.to_string(),
                episode.season_id.to_string(),
                episode.season_number,
                episode.episode_number,
                episode.title,
                episode.overview,
                episode.still_path,
                episode.air_date,
                episode.duration_seconds,
                episode.rating,
                episode.created_at.to_rfc3339(),
                episode.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_episode(&self, id: &Uuid) -> AppResult<Option<TvEpisode>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, series_id, season_id, season_number, episode_number, title, overview, still_path, air_date, duration_seconds, rating, created_at, updated_at FROM tv_episode WHERE id = ?1"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![id.to_string()]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(Self::row_to_episode(row).map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn list_episodes_by_season(&self, season_id: &Uuid) -> AppResult<Vec<TvEpisode>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, series_id, season_id, season_number, episode_number, title, overview, still_path, air_date, duration_seconds, rating, created_at, updated_at FROM tv_episode WHERE season_id = ?1 ORDER BY episode_number ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![season_id.to_string()], |row| Self::row_to_episode(row))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    fn get_series_details(&self, series_id: &Uuid) -> AppResult<Option<SeriesDetails>> {
        let series = match self.get_series(series_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        let seasons = self.list_seasons_by_series(series_id)?;
        let mut season_details = Vec::new();
        let mut total_episodes = 0;

        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        for season in seasons {
            let mut ep_stmt = conn.prepare(
                "
                SELECT e.id, e.series_id, e.season_id, e.season_number, e.episode_number,
                       e.title, e.overview, e.still_path, e.air_date, e.duration_seconds,
                       e.rating, e.created_at, e.updated_at,
                       m.path, m.id,
                       COALESCE(p.position_seconds, 0),
                       COALESCE(p.duration_seconds, e.duration_seconds, 0),
                       COALESCE(p.completed, 0)
                FROM tv_episode e
                LEFT JOIN media m ON m.episode_id = e.id
                LEFT JOIN playback_progress p ON p.episode_id = e.id
                WHERE e.season_id = ?1
                ORDER BY e.episode_number ASC
                "
            ).map_err(|e| AppError::Database(e.to_string()))?;

            let ep_rows = ep_stmt.query_map(params![season.id.to_string()], |row| {
                let ep = Self::row_to_episode(row)?;
                let media_path: Option<String> = row.get(13)?;
                let media_id_str: Option<String> = row.get(14)?;
                let media_id = media_id_str.and_then(|s| Uuid::parse_str(&s).ok());
                let progress_seconds: u32 = row.get(15)?;
                let duration_seconds: u32 = row.get(16)?;
                let completed_int: i32 = row.get(17)?;

                Ok(EpisodeWithMedia {
                    episode: ep,
                    media_path,
                    media_id,
                    progress_seconds,
                    duration_seconds,
                    completed: completed_int == 1,
                })
            }).map_err(|e| AppError::Database(e.to_string()))?;

            let mut episodes = Vec::new();
            for e in ep_rows {
                episodes.push(e.map_err(|e| AppError::Database(e.to_string()))?);
            }
            total_episodes += episodes.len() as u32;

            season_details.push(SeasonWithEpisodes {
                season,
                episodes,
            });
        }

        Ok(Some(SeriesDetails {
            series,
            seasons: season_details,
            total_episodes,
        }))
    }

    fn get_next_episode(&self, series_id: &Uuid, season_number: u32, episode_number: u32) -> AppResult<Option<TvEpisode>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        // 1. Try next episode in current season
        let mut stmt = conn.prepare(
            "
            SELECT id, series_id, season_id, season_number, episode_number, title, overview, still_path, air_date, duration_seconds, rating, created_at, updated_at
            FROM tv_episode
            WHERE series_id = ?1 AND season_number = ?2 AND episode_number = ?3
            LIMIT 1
            "
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![series_id.to_string(), season_number, episode_number + 1])
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            return Ok(Some(Self::row_to_episode(row).map_err(|e| AppError::Database(e.to_string()))?));
        }

        // 2. Try episode 1 of next season
        let mut next_season_stmt = conn.prepare(
            "
            SELECT id, series_id, season_id, season_number, episode_number, title, overview, still_path, air_date, duration_seconds, rating, created_at, updated_at
            FROM tv_episode
            WHERE series_id = ?1 AND season_number = ?2 AND episode_number = 1
            LIMIT 1
            "
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut next_rows = next_season_stmt.query(params![series_id.to_string(), season_number + 1])
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = next_rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            return Ok(Some(Self::row_to_episode(row).map_err(|e| AppError::Database(e.to_string()))?));
        }

        Ok(None)
    }
}
