use std::sync::Arc;

use crate::abstractions::SettingsRepository;
use crate::domain::AppSettings;
use crate::error::AppResult;

pub struct SettingsService {
    settings_repo: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    pub fn new(settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { settings_repo }
    }

    pub fn get(&self) -> AppResult<AppSettings> {
        self.settings_repo.get_settings()
    }

    pub fn update(&self, new_settings: AppSettings) -> AppResult<AppSettings> {
        self.settings_repo.save_settings(&new_settings)?;
        Ok(new_settings)
    }
}
