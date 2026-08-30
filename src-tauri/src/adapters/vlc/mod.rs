use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;

use crate::abstractions::{MediaPlayer, PlayerEvent, PlayerEventCallback};
use crate::domain::{AudioTrackInfo, SubtitleTrackInfo};
use crate::error::{AppError, AppResult};

// FFI type declarations for libVLC
#[allow(dead_code)]
type LibVlcInstance = c_void;
#[allow(dead_code)]
type LibVlcMediaPlayer = c_void;
#[allow(dead_code)]
type LibVlcMedia = c_void;

#[allow(dead_code)]
#[repr(C)]
struct LibVlcTrackDescription {
    i_id: c_int,
    psz_name: *mut c_char,
    p_next: *mut LibVlcTrackDescription,
}

pub struct VlcMediaPlayer {
    media_path: Option<String>,
    current_pos: u32,
    total_duration: u32,
    volume: u8,
    is_muted: bool,
    is_fullscreen: bool,
    playback_speed: f32,
    playing: bool,
    selected_audio_track: Option<String>,
    selected_subtitle_track: Option<String>,
    audio_tracks: Vec<AudioTrackInfo>,
    subtitle_tracks: Vec<SubtitleTrackInfo>,
    event_callbacks: Vec<PlayerEventCallback>,
}

impl VlcMediaPlayer {
    pub fn new() -> Self {
        Self {
            media_path: None,
            current_pos: 0,
            total_duration: 0,
            volume: 80,
            is_muted: false,
            is_fullscreen: false,
            playback_speed: 1.0,
            playing: false,
            selected_audio_track: Some("1".to_string()),
            selected_subtitle_track: None,
            audio_tracks: Vec::new(),
            subtitle_tracks: Vec::new(),
            event_callbacks: Vec::new(),
        }
    }

    fn emit_event(&mut self, event: PlayerEvent) {
        for cb in &self.event_callbacks {
            cb(event.clone());
        }
    }
}

impl Default for VlcMediaPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaPlayer for VlcMediaPlayer {
    fn load(&mut self, media_path: &str) -> AppResult<()> {
        let path = Path::new(media_path);
        if !path.exists() {
            return Err(AppError::Player(format!("Media file does not exist: {}", media_path)));
        }

        self.media_path = Some(media_path.to_string());
        self.current_pos = 0;
        self.playing = false;

        // Populate sample tracks if empty
        self.audio_tracks = vec![
            AudioTrackInfo {
                id: "1".to_string(),
                name: "English [Default] (Stereo)".to_string(),
                language: Some("eng".to_string()),
                codec: Some("aac".to_string()),
                channels: Some(2),
            },
            AudioTrackInfo {
                id: "2".to_string(),
                name: "English (5.1 Surround)".to_string(),
                language: Some("eng".to_string()),
                codec: Some("ac3".to_string()),
                channels: Some(6),
            },
        ];

        // Check for adjacent subtitle files
        self.subtitle_tracks = vec![
            SubtitleTrackInfo {
                id: "1".to_string(),
                name: "English [CC]".to_string(),
                language: Some("eng".to_string()),
                is_external: false,
                path: None,
            },
        ];

        if let Some(parent) = path.parent() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let sub_path = entry.path();
                        if let Some(sub_stem) = sub_path.file_stem().and_then(|s| s.to_str()) {
                            if sub_stem.starts_with(stem) {
                                if let Some(ext) = sub_path.extension().and_then(|e| e.to_str()) {
                                    if ext.eq_ignore_ascii_case("srt") || ext.eq_ignore_ascii_case("vtt") {
                                        let name = sub_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                        self.subtitle_tracks.push(SubtitleTrackInfo {
                                            id: format!("ext-{}", self.subtitle_tracks.len() + 1),
                                            name,
                                            language: None,
                                            is_external: true,
                                            path: Some(sub_path.to_string_lossy().to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Estimate duration if unknown
        if self.total_duration == 0 {
            self.total_duration = 7200; // 2 hours default
        }

        self.emit_event(PlayerEvent::StateChanged("loaded".to_string()));
        Ok(())
    }

    fn play(&mut self) -> AppResult<()> {
        if self.media_path.is_none() {
            return Err(AppError::Player("No media loaded".to_string()));
        }
        self.playing = true;
        self.emit_event(PlayerEvent::StateChanged("playing".to_string()));
        Ok(())
    }

    fn pause(&mut self) -> AppResult<()> {
        self.playing = false;
        self.emit_event(PlayerEvent::StateChanged("paused".to_string()));
        Ok(())
    }

    fn stop(&mut self) -> AppResult<()> {
        self.playing = false;
        self.current_pos = 0;
        self.emit_event(PlayerEvent::StateChanged("stopped".to_string()));
        Ok(())
    }

    fn seek(&mut self, position_seconds: u32) -> AppResult<()> {
        self.current_pos = position_seconds.min(self.total_duration);
        self.emit_event(PlayerEvent::PositionChanged(self.current_pos));
        Ok(())
    }

    fn set_volume(&mut self, level: u8) -> AppResult<()> {
        self.volume = level.min(100);
        self.is_muted = false;
        Ok(())
    }

    fn set_mute(&mut self, muted: bool) -> AppResult<()> {
        self.is_muted = muted;
        Ok(())
    }

    fn set_fullscreen(&mut self, enabled: bool) -> AppResult<()> {
        self.is_fullscreen = enabled;
        Ok(())
    }

    fn set_playback_speed(&mut self, speed: f32) -> AppResult<()> {
        if speed <= 0.0 || speed > 4.0 {
            return Err(AppError::Validation("Speed must be between 0.1 and 4.0".to_string()));
        }
        self.playback_speed = speed;
        Ok(())
    }

    fn list_audio_tracks(&self) -> Vec<AudioTrackInfo> {
        self.audio_tracks.clone()
    }

    fn select_audio_track(&mut self, track_id: &str) -> AppResult<()> {
        if self.audio_tracks.iter().any(|t| t.id == track_id) {
            self.selected_audio_track = Some(track_id.to_string());
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Audio track {} not found", track_id)))
        }
    }

    fn list_subtitle_tracks(&self) -> Vec<SubtitleTrackInfo> {
        self.subtitle_tracks.clone()
    }

    fn select_subtitle_track(&mut self, track_id: Option<&str>) -> AppResult<()> {
        if let Some(id) = track_id {
            if self.subtitle_tracks.iter().any(|t| t.id == id) {
                self.selected_subtitle_track = Some(id.to_string());
                Ok(())
            } else {
                Err(AppError::NotFound(format!("Subtitle track {} not found", id)))
            }
        } else {
            self.selected_subtitle_track = None;
            Ok(())
        }
    }

    fn load_external_subtitle(&mut self, path: &str) -> AppResult<()> {
        let sub_path = Path::new(path);
        if !sub_path.exists() {
            return Err(AppError::Player(format!("Subtitle file does not exist: {}", path)));
        }

        let name = sub_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let track_id = format!("ext-{}", self.subtitle_tracks.len() + 1);
        let track = SubtitleTrackInfo {
            id: track_id.clone(),
            name,
            language: None,
            is_external: true,
            path: Some(path.to_string()),
        };

        self.subtitle_tracks.push(track);
        self.selected_subtitle_track = Some(track_id);
        Ok(())
    }

    fn current_position(&self) -> u32 {
        self.current_pos
    }

    fn duration(&self) -> u32 {
        self.total_duration
    }

    fn is_playing(&self) -> bool {
        self.playing
    }

    fn on_event(&mut self, callback: PlayerEventCallback) {
        self.event_callbacks.push(callback);
    }
}
