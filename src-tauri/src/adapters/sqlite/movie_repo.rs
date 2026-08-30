use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::abstractions::MovieRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{MetadataStatus, Movie, MovieFilter, MovieSort};
use crate::error::{AppError, AppResult};

pub struct SqliteMovieRepository {
    db: SqliteDb,
}

impl SqliteMovieRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_movie(row: &Row) -> Result<Movie, rusqlite::Error> {
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
    }
}

impl MovieRepository for SqliteMovieRepository {
    fn upsert_movie(&self, movie: &Movie) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let genres_json = serde_json::to_string(&movie.genres).unwrap_or_else(|_| "[]".to_string());
        let cast_json = serde_json::to_string(&movie.cast).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "
            INSERT INTO movie (
                id, title, original_title, year, description, poster_path, backdrop_path,
                genres, [cast], director, rating, metadata_provider_id, metadata_status,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                original_title = excluded.original_title,
                year = excluded.year,
                description = excluded.description,
                poster_path = excluded.poster_path,
                backdrop_path = excluded.backdrop_path,
                genres = excluded.genres,
                [cast] = excluded.[cast],
                director = excluded.director,
                rating = excluded.rating,
                metadata_provider_id = excluded.metadata_provider_id,
                metadata_status = excluded.metadata_status,
                updated_at = excluded.updated_at
            ",
            params![
                movie.id.to_string(),
                movie.title,
                movie.original_title,
                movie.year,
                movie.description,
                movie.poster_path,
                movie.backdrop_path,
                genres_json,
                cast_json,
                movie.director,
                movie.rating,
                movie.metadata_provider_id,
                movie.metadata_status.to_string(),
                movie.created_at.to_rfc3339(),
                movie.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    fn get_movie(&self, id: &Uuid) -> AppResult<Option<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT id, title, original_title, year, description, poster_path, backdrop_path,
                   genres, [cast], director, rating, metadata_provider_id, metadata_status,
                   created_at, updated_at
            FROM movie
            WHERE id = ?1
            ",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_movie(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_title_year(&self, title: &str, year: Option<u16>) -> AppResult<Option<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let sql = if year.is_some() {
            "
            SELECT id, title, original_title, year, description, poster_path, backdrop_path,
                   genres, [cast], director, rating, metadata_provider_id, metadata_status,
                   created_at, updated_at
            FROM movie
            WHERE LOWER(title) = LOWER(?1) AND year = ?2
            LIMIT 1
            "
        } else {
            "
            SELECT id, title, original_title, year, description, poster_path, backdrop_path,
                   genres, [cast], director, rating, metadata_provider_id, metadata_status,
                   created_at, updated_at
            FROM movie
            WHERE LOWER(title) = LOWER(?1) AND year IS NULL
            LIMIT 1
            "
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = if let Some(y) = year {
            stmt.query(params![title, y])?
        } else {
            stmt.query(params![title])?
        };

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_movie(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_provider_id(&self, provider_id: &str) -> AppResult<Option<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "
            SELECT id, title, original_title, year, description, poster_path, backdrop_path,
                   genres, [cast], director, rating, metadata_provider_id, metadata_status,
                   created_at, updated_at
            FROM movie
            WHERE metadata_provider_id = ?1
            LIMIT 1
            ",
        )?;

        let mut rows = stmt.query(params![provider_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_movie(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_movies(&self, filter: &MovieFilter, sort: MovieSort) -> AppResult<Vec<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut query = String::from(
            "
            SELECT DISTINCT m.id, m.title, m.original_title, m.year, m.description, m.poster_path, m.backdrop_path,
                   m.genres, m.[cast], m.director, m.rating, m.metadata_provider_id, m.metadata_status,
                   m.created_at, m.updated_at
            FROM movie m
            LEFT JOIN media med ON med.movie_id = m.id
            LEFT JOIN watchlist w ON w.movie_id = m.id
            LEFT JOIN playback_state ps ON ps.movie_id = m.id
            LEFT JOIN watch_history wh ON wh.movie_id = m.id
            WHERE 1=1
            ",
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref genre) = filter.genre {
            query.push_str(" AND m.genres LIKE ?");
            params_vec.push(Box::new(format!("%\"{}\"%", genre)));
        }

        if let Some(year) = filter.year {
            query.push_str(" AND m.year = ?");
            params_vec.push(Box::new(year));
        }

        if let Some(in_wl) = filter.in_watchlist {
            if in_wl {
                query.push_str(" AND w.movie_id IS NOT NULL");
            } else {
                query.push_str(" AND w.movie_id IS NULL");
            }
        }

        if let Some(watched) = filter.watched {
            if watched {
                query.push_str(" AND (ps.completed = 1 OR wh.completed_at IS NOT NULL)");
            } else {
                query.push_str(" AND (ps.completed IS NULL OR ps.completed = 0) AND wh.completed_at IS NULL");
            }
        }

        if let Some(source_id) = filter.source_id {
            query.push_str(" AND med.source_id = ?");
            params_vec.push(Box::new(source_id.to_string()));
        }

        if let Some(min_rating) = filter.min_rating {
            query.push_str(" AND m.rating >= ?");
            params_vec.push(Box::new(min_rating));
        }

        if let Some(is_avail) = filter.is_available {
            if is_avail {
                query.push_str(" AND med.availability = 'available'");
            } else {
                query.push_str(" AND (med.availability = 'unavailable' OR med.id IS NULL)");
            }
        }

        match sort {
            MovieSort::TitleAsc => query.push_str(" ORDER BY m.title ASC"),
            MovieSort::TitleDesc => query.push_str(" ORDER BY m.title DESC"),
            MovieSort::YearAsc => query.push_str(" ORDER BY m.year ASC NULLS LAST, m.title ASC"),
            MovieSort::YearDesc => query.push_str(" ORDER BY m.year DESC NULLS LAST, m.title ASC"),
            MovieSort::DateAddedAsc => query.push_str(" ORDER BY m.created_at ASC"),
            MovieSort::DateAddedDesc => query.push_str(" ORDER BY m.created_at DESC"),
            MovieSort::RecentlyWatched => {
                query.push_str(" ORDER BY ps.updated_at DESC NULLS LAST, m.updated_at DESC")
            }
            MovieSort::RatingDesc => query.push_str(" ORDER BY m.rating DESC NULLS LAST, m.title ASC"),
        }

        let mut stmt = conn.prepare(&query)?;
        let rusqlite_params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(&rusqlite_params[..], Self::row_to_movie)?;

        let mut movies = Vec::new();
        for movie_res in rows {
            movies.push(movie_res?);
        }

        Ok(movies)
    }

    fn search_movies(&self, query: &str) -> AppResult<Vec<Movie>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let pattern = format!("%{}%", query.trim());

        let mut stmt = conn.prepare(
            "
            SELECT id, title, original_title, year, description, poster_path, backdrop_path,
                   genres, [cast], director, rating, metadata_provider_id, metadata_status,
                   created_at, updated_at
            FROM movie
            WHERE title LIKE ?1
               OR original_title LIKE ?1
               OR director LIKE ?1
               OR [cast] LIKE ?1
               OR genres LIKE ?1
               OR description LIKE ?1
               OR CAST(year AS TEXT) LIKE ?1
            ORDER BY
                CASE
                    WHEN LOWER(title) = LOWER(?2) THEN 1
                    WHEN LOWER(title) LIKE LOWER(?3) THEN 2
                    ELSE 3
                END,
                title ASC
            ",
        )?;

        let prefix_pattern = format!("{}%", query.trim());
        let exact_pattern = query.trim().to_string();

        let rows = stmt.query_map(
            params![pattern, exact_pattern, prefix_pattern],
            Self::row_to_movie,
        )?;

        let mut movies = Vec::new();
        for movie_res in rows {
            movies.push(movie_res?);
        }

        Ok(movies)
    }

    fn delete_movie(&self, id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("DELETE FROM movie WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn count_movies(&self) -> AppResult<u32> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let count: u32 = conn.query_row("SELECT COUNT(*) FROM movie", [], |r| r.get(0))?;
        Ok(count)
    }
}
