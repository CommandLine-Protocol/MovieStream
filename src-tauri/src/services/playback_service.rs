use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::abstractions::{
    MediaPlayer, PlaybackStateRepository, ProgressRepository, TvRepository, WatchHistoryRepository,
};
use crate::domain::{
    AudioTrackInfo, MediaProgress, MediaType, PlaybackState, SubtitleTrackInfo, TvEpisode,
    WatchHistoryEntry, DEFAULT_COMPLETION_THRESHOLD,
};
use crate::error::{AppError, AppResult};
use crate::services::stream_server::MediaStreamServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSession {
    pub media_type: String, // "movie" | "episode"
    pub media_id: Uuid,
    pub movie_id: Option<Uuid>,
    pub series_id: Option<Uuid>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub episode_id: Option<Uuid>,
    pub title: String,
    pub subtitle_info: Option<String>,
    pub media_path: String,
    pub stream_url: String,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub is_playing: bool,
    pub is_fullscreen: bool,
    pub volume: u8,
    pub is_muted: bool,
    pub playback_speed: f32,
    pub current_audio_track: Option<String>,
    pub current_subtitle_track: Option<String>,
    pub audio_tracks: Vec<AudioTrackInfo>,
    pub subtitle_tracks: Vec<SubtitleTrackInfo>,
    pub requires_resume_prompt: bool,
    pub resume_position_seconds: u32,
}

pub struct PlaybackService {
    player: Arc<Mutex<Box<dyn MediaPlayer>>>,
    playback_repo: Arc<dyn PlaybackStateRepository>,
    history_repo: Arc<dyn WatchHistoryRepository>,
    progress_repo: Option<Arc<dyn ProgressRepository>>,
    tv_repo: Option<Arc<dyn TvRepository>>,
    stream_server: Option<Arc<MediaStreamServer>>,
    active_session: Arc<Mutex<Option<PlaybackSession>>>,
    completion_threshold: f32,
}

impl PlaybackService {
    pub fn new(
        player: Arc<Mutex<Box<dyn MediaPlayer>>>,
        playback_repo: Arc<dyn PlaybackStateRepository>,
        history_repo: Arc<dyn WatchHistoryRepository>,
    ) -> Self {
        Self {
            player,
            playback_repo,
            history_repo,
            progress_repo: None,
            tv_repo: None,
            stream_server: None,
            active_session: Arc::new(Mutex::new(None)),
            completion_threshold: DEFAULT_COMPLETION_THRESHOLD,
        }
    }

    pub fn with_full_engine(
        player: Arc<Mutex<Box<dyn MediaPlayer>>>,
        playback_repo: Arc<dyn PlaybackStateRepository>,
        history_repo: Arc<dyn WatchHistoryRepository>,
        progress_repo: Arc<dyn ProgressRepository>,
        tv_repo: Arc<dyn TvRepository>,
        stream_server: Arc<MediaStreamServer>,
    ) -> Self {
        Self {
            player,
            playback_repo,
            history_repo,
            progress_repo: Some(progress_repo),
            tv_repo: Some(tv_repo),
            stream_server: Some(stream_server),
            active_session: Arc::new(Mutex::new(None)),
            completion_threshold: DEFAULT_COMPLETION_THRESHOLD,
        }
    }

    pub fn set_completion_threshold(&mut self, threshold: f32) {
        self.completion_threshold = threshold;
    }

    pub fn start_movie(&self, movie_id: Uuid, media_id: Uuid, media_path: &str, title: &str) -> AppResult<PlaybackSession> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.load(media_path)?;

        let duration = player.duration();

        // Check saved progress
        let saved_progress = if let Some(ref p_repo) = self.progress_repo {
            p_repo.get_progress_by_movie(&movie_id)?
        } else {
            None
        };

        let (requires_resume_prompt, resume_position) = match saved_progress {
            Some(ref p) if p.position_seconds > 10 && !p.completed => {
                (true, p.position_seconds)
            }
            _ => {
                // Fallback to legacy playback state
                if let Ok(Some(state)) = self.playback_repo.get_state(&movie_id) {
                    if state.position_seconds > 10 && !state.completed {
                        (true, state.position_seconds)
                    } else {
                        (false, 0)
                    }
                } else {
                    (false, 0)
                }
            }
        };

        let audio_tracks = player.list_audio_tracks();
        let subtitle_tracks = player.list_subtitle_tracks();

        let stream_url = if let Some(ref srv) = self.stream_server {
            srv.get_stream_url(media_path)
        } else {
            media_path.to_string()
        };

        let mut mapped_subtitles = Vec::new();
        for mut sub in subtitle_tracks {
            if let Some(ref p) = sub.path {
                if let Some(ref srv) = self.stream_server {
                    sub.path = Some(srv.get_subtitle_url(p));
                }
            }
            mapped_subtitles.push(sub);
        }

        let session = PlaybackSession {
            media_type: "movie".to_string(),
            media_id,
            movie_id: Some(movie_id),
            series_id: None,
            season_number: None,
            episode_number: None,
            episode_id: None,
            title: title.to_string(),
            subtitle_info: None,
            media_path: media_path.to_string(),
            stream_url,
            position_seconds: 0,
            duration_seconds: duration,
            is_playing: false,
            is_fullscreen: false,
            volume: 80,
            is_muted: false,
            playback_speed: 1.0,
            current_audio_track: audio_tracks.first().map(|t| t.id.clone()),
            current_subtitle_track: None,
            audio_tracks,
            subtitle_tracks: mapped_subtitles,
            requires_resume_prompt,
            resume_position_seconds: resume_position,
        };

        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        *active = Some(session.clone());

        // Create initial history record
        let history_entry = WatchHistoryEntry {
            id: Uuid::new_v4(),
            movie_id,
            started_at: Utc::now(),
            completed_at: None,
            last_position_seconds: 0,
        };
        let _ = self.history_repo.add_entry(&history_entry);

        if !requires_resume_prompt {
            player.play()?;
        }

        Ok(session)
    }

    pub fn start_episode(
        &self,
        episode_id: Uuid,
        media_id: Uuid,
        media_path: &str,
        series_id: Uuid,
        season_number: u32,
        episode_number: u32,
        series_title: &str,
        episode_title: &str,
    ) -> AppResult<PlaybackSession> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.load(media_path)?;

        let duration = player.duration();

        // Check saved progress
        let saved_progress = if let Some(ref p_repo) = self.progress_repo {
            p_repo.get_progress_by_episode(&episode_id)?
        } else {
            None
        };

        let (requires_resume_prompt, resume_position) = match saved_progress {
            Some(ref p) if p.position_seconds > 10 && !p.completed => {
                (true, p.position_seconds)
            }
            _ => (false, 0),
        };

        let audio_tracks = player.list_audio_tracks();
        let subtitle_tracks = player.list_subtitle_tracks();

        let stream_url = if let Some(ref srv) = self.stream_server {
            srv.get_stream_url(media_path)
        } else {
            media_path.to_string()
        };

        let mut mapped_subtitles = Vec::new();
        for mut sub in subtitle_tracks {
            if let Some(ref p) = sub.path {
                if let Some(ref srv) = self.stream_server {
                    sub.path = Some(srv.get_subtitle_url(p));
                }
            }
            mapped_subtitles.push(sub);
        }

        let session = PlaybackSession {
            media_type: "episode".to_string(),
            media_id,
            movie_id: None,
            series_id: Some(series_id),
            season_number: Some(season_number),
            episode_number: Some(episode_number),
            episode_id: Some(episode_id),
            title: series_title.to_string(),
            subtitle_info: Some(format!("S{:02}E{:02} • {}", season_number, episode_number, episode_title)),
            media_path: media_path.to_string(),
            stream_url,
            position_seconds: 0,
            duration_seconds: duration,
            is_playing: false,
            is_fullscreen: false,
            volume: 80,
            is_muted: false,
            playback_speed: 1.0,
            current_audio_track: audio_tracks.first().map(|t| t.id.clone()),
            current_subtitle_track: None,
            audio_tracks,
            subtitle_tracks: mapped_subtitles,
            requires_resume_prompt,
            resume_position_seconds: resume_position,
        };

        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        *active = Some(session.clone());

        if !requires_resume_prompt {
            player.play()?;
        }

        Ok(session)
    }

    pub fn start(&self, movie_id: Uuid, media_id: Uuid, media_path: &str) -> AppResult<PlaybackSession> {
        self.start_movie(movie_id, media_id, media_path, "Movie")
    }

    pub fn play(&self) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.play()?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.is_playing = true;
        }
        Ok(())
    }

    pub fn pause(&self) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.pause()?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.is_playing = false;
            self.persist_session_progress(session);
        }
        Ok(())
    }

    pub fn stop(&self) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.stop()?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.is_playing = false;
            self.persist_session_progress(session);
        }
        *active = None;
        Ok(())
    }

    pub fn seek(&self, position_seconds: u32) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.seek(position_seconds)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.position_seconds = position_seconds;
            self.persist_session_progress(session);
        }
        Ok(())
    }

    pub fn resume_from(&self, position_seconds: u32) -> AppResult<()> {
        self.seek(position_seconds)?;
        self.play()
    }

    pub fn set_volume(&self, level: u8) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.set_volume(level)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.volume = level;
            session.is_muted = false;
        }
        Ok(())
    }

    pub fn set_mute(&self, muted: bool) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.set_mute(muted)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.is_muted = muted;
        }
        Ok(())
    }

    pub fn set_fullscreen(&self, enabled: bool) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.set_fullscreen(enabled)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.is_fullscreen = enabled;
        }
        Ok(())
    }

    pub fn set_playback_speed(&self, speed: f32) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.set_playback_speed(speed)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.playback_speed = speed;
        }
        Ok(())
    }

    pub fn select_audio_track(&self, track_id: &str) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.select_audio_track(track_id)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.current_audio_track = Some(track_id.to_string());
        }
        Ok(())
    }

    pub fn select_subtitle_track(&self, track_id: Option<&str>) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.select_subtitle_track(track_id)?;
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.current_subtitle_track = track_id.map(|s| s.to_string());
        }
        Ok(())
    }

    pub fn load_external_subtitle(&self, path: &str) -> AppResult<()> {
        let mut player = self.player.lock().map_err(|e| AppError::Player(e.to_string()))?;
        player.load_external_subtitle(path)?;
        Ok(())
    }

    pub fn get_active_session(&self) -> Option<PlaybackSession> {
        let active = self.active_session.lock().ok()?;
        active.clone()
    }

    pub fn record_position_tick(&self, position_seconds: u32) -> AppResult<()> {
        let mut active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref mut session) = *active {
            session.position_seconds = position_seconds;
            self.persist_session_progress(session);
        }
        Ok(())
    }

    pub fn get_next_episode(&self) -> AppResult<Option<TvEpisode>> {
        let active = self.active_session.lock().map_err(|e| AppError::Player(e.to_string()))?;
        if let Some(ref session) = *active {
            if let (Some(series_id), Some(season_num), Some(ep_num), Some(ref tv_repo)) = (
                session.series_id,
                session.season_number,
                session.episode_number,
                self.tv_repo.as_ref(),
            ) {
                return tv_repo.get_next_episode(&series_id, season_num, ep_num);
            }
        }
        Ok(None)
    }

    fn persist_session_progress(&self, session: &PlaybackSession) {
        let is_completed = if session.duration_seconds > 0 {
            let threshold = session.duration_seconds as f32 * self.completion_threshold;
            let near_end = session.duration_seconds.saturating_sub(session.position_seconds) <= 90;
            (session.position_seconds as f32 >= threshold) || near_end
        } else {
            false
        };

        let now = Utc::now();

        // 1. Save unified progress
        if let Some(ref p_repo) = self.progress_repo {
            let percentage = if session.duration_seconds > 0 {
                (session.position_seconds as f32 / session.duration_seconds as f32) * 100.0
            } else {
                0.0
            };

            let progress = MediaProgress {
                id: Uuid::new_v4(),
                media_type: if session.media_type == "episode" {
                    MediaType::Episode
                } else {
                    MediaType::Movie
                },
                media_id: session.media_id,
                movie_id: session.movie_id,
                series_id: session.series_id,
                season_number: session.season_number,
                episode_number: session.episode_number,
                episode_id: session.episode_id,
                position_seconds: if is_completed { 0 } else { session.position_seconds },
                duration_seconds: session.duration_seconds,
                progress_percentage: if is_completed { 100.0 } else { percentage },
                completed: is_completed,
                last_watched: now,
            };

            let _ = p_repo.save_progress(&progress);
        }

        // 2. Legacy movie support
        if let Some(movie_id) = session.movie_id {
            let state = PlaybackState {
                movie_id,
                media_id: session.media_id,
                position_seconds: if is_completed { 0 } else { session.position_seconds },
                duration_seconds: session.duration_seconds,
                completed: is_completed,
                updated_at: now,
            };
            let _ = self.playback_repo.upsert_state(&state);
        }
    }
}
