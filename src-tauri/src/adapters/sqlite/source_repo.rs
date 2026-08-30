use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::abstractions::LibrarySourceRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::{LibrarySource, SourceStatus};
use crate::error::{AppError, AppResult};

pub struct SqliteLibrarySourceRepository {
    db: SqliteDb,
}

impl SqliteLibrarySourceRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    fn row_to_source(row: &Row) -> Result<LibrarySource, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let path: String = row.get(1)?;
        let name: String = row.get(2)?;
        let status_str: String = row.get(3)?;
        let last_scanned_str: Option<String> = row.get(4)?;
        let created_at_str: String = row.get(5)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let status = SourceStatus::from_str(&status_str).unwrap_or_default();
        let last_scanned_at = last_scanned_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(LibrarySource {
            id,
            path,
            name,
            status,
            last_scanned_at,
            created_at,
        })
    }
}

impl LibrarySourceRepository for SqliteLibrarySourceRepository {
    fn upsert_source(&self, source: &LibrarySource) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let last_scanned = source.last_scanned_at.map(|t| t.to_rfc3339());

        conn.execute(
            "
            INSERT INTO library_source (id, path, name, status, last_scanned_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                status = excluded.status,
                last_scanned_at = excluded.last_scanned_at
            ",
            params![
                source.id.to_string(),
                source.path,
                source.name,
                source.status.to_string(),
                last_scanned,
                source.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    fn get_source(&self, id: &Uuid) -> AppResult<Option<LibrarySource>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, path, name, status, last_scanned_at, created_at FROM library_source WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_source(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_path(&self, path: &str) -> AppResult<Option<LibrarySource>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, path, name, status, last_scanned_at, created_at FROM library_source WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_source(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_sources(&self) -> AppResult<Vec<LibrarySource>> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, path, name, status, last_scanned_at, created_at FROM library_source ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([], Self::row_to_source)?;
        let mut sources = Vec::new();
        for source_res in rows {
            sources.push(source_res?);
        }

        Ok(sources)
    }

    fn set_status(&self, id: &Uuid, status: SourceStatus) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "UPDATE library_source SET status = ?1 WHERE id = ?2",
            params![status.to_string(), id.to_string()],
        )?;

        Ok(())
    }

    fn update_last_scanned(&self, id: &Uuid, time: DateTime<Utc>) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "UPDATE library_source SET last_scanned_at = ?1 WHERE id = ?2",
            params![time.to_rfc3339(), id.to_string()],
        )?;

        Ok(())
    }

    fn delete_source(&self, id: &Uuid) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("DELETE FROM library_source WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }
}
