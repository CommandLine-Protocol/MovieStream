use crate::domain::{AudioTrackInfo, SubtitleTrackInfo};
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    PositionChanged(u32),
    StateChanged(String), // "playing" | "paused" | "stopped" | "ended"
    Error(String),
    Finished,
}

pub type PlayerEventCallback = Box<dyn Fn(PlayerEvent) + Send + Sync>;

pub trait MediaPlayer: Send + Sync {
    fn load(&mut self, media_path: &str) -> AppResult<()>;
    fn play(&mut self) -> AppResult<()>;
    fn pause(&mut self) -> AppResult<()>;
    fn stop(&mut self) -> AppResult<()>;
    fn seek(&mut self, position_seconds: u32) -> AppResult<()>;
    fn set_volume(&mut self, level: u8) -> AppResult<()>;
    fn set_mute(&mut self, muted: bool) -> AppResult<()>;
    fn set_fullscreen(&mut self, enabled: bool) -> AppResult<()>;
    fn set_playback_speed(&mut self, speed: f32) -> AppResult<()>;

    fn list_audio_tracks(&self) -> Vec<AudioTrackInfo>;
    fn select_audio_track(&mut self, track_id: &str) -> AppResult<()>;

    fn list_subtitle_tracks(&self) -> Vec<SubtitleTrackInfo>;
    fn select_subtitle_track(&mut self, track_id: Option<&str>) -> AppResult<()>;
    fn load_external_subtitle(&mut self, path: &str) -> AppResult<()>;

    fn current_position(&self) -> u32;
    fn duration(&self) -> u32;
    fn is_playing(&self) -> bool;

    fn on_event(&mut self, callback: PlayerEventCallback);
}
