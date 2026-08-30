use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibrarySettings {
    pub scan_on_startup: bool,
    pub default_locations: Vec<String>,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            scan_on_startup: true,
            default_locations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaybackSettings {
    pub default_volume: u8,
    pub default_speed: f32,
    pub resume_behavior: String, // "prompt", "always", "never"
    pub completion_threshold: f32, // default 0.90
    pub audio_language_preference: Option<String>,
    pub subtitle_language_preference: Option<String>,
    pub subtitles_enabled_by_default: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            default_volume: 80,
            default_speed: 1.0,
            resume_behavior: "prompt".to_string(),
            completion_threshold: 0.90,
            audio_language_preference: None,
            subtitle_language_preference: None,
            subtitles_enabled_by_default: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceSettings {
    pub theme: String, // "dark", "cinematic", "midnight"
    pub animations_enabled: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "cinematic".to_string(),
            animations_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataSettings {
    pub active_provider_id: String, // "tmdb", "open_movie", "mock"
    pub artwork_caching_enabled: bool,
    pub auto_match_threshold: f32,
}

impl Default for MetadataSettings {
    fn default() -> Self {
        Self {
            active_provider_id: "open_movie".to_string(),
            artwork_caching_enabled: true,
            auto_match_threshold: 0.8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplicationSettings {
    pub launch_on_startup: bool,
    pub notifications_enabled: bool,
    pub log_level: String,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            launch_on_startup: false,
            notifications_enabled: true,
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub library: LibrarySettings,
    pub playback: PlaybackSettings,
    pub appearance: AppearanceSettings,
    pub metadata: MetadataSettings,
    pub application: ApplicationSettings,
}
