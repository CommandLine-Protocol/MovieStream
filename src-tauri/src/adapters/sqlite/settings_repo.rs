use rusqlite::params;

use crate::abstractions::SettingsRepository;
use crate::adapters::sqlite::db::SqliteDb;
use crate::domain::AppSettings;
use crate::error::{AppError, AppResult};

pub struct SqliteSettingsRepository {
    db: SqliteDb,
}

impl SqliteSettingsRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }
}

const SETTINGS_KEY: &str = "global_settings";

impl SettingsRepository for SqliteSettingsRepository {
    fn get_settings(&self) -> AppResult<AppSettings> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![SETTINGS_KEY])?;

        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let settings: AppSettings = serde_json::from_str(&json_str)
                .unwrap_or_else(|_| AppSettings::default());
            Ok(settings)
        } else {
            let default_settings = AppSettings::default();
            let json_str = serde_json::to_string(&default_settings).unwrap_or_default();
            conn.execute(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
                params![SETTINGS_KEY, json_str],
            )?;
            Ok(default_settings)
        }
    }

    fn save_settings(&self, settings: &AppSettings) -> AppResult<()> {
        let conn = self.db.conn();
        let conn = conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let json_str = serde_json::to_string(settings)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        conn.execute(
            "
            INSERT INTO app_settings (key, value) VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            ",
            params![SETTINGS_KEY, json_str],
        )?;

        Ok(())
    }
}
