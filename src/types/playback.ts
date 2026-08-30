import { AudioTrackInfo, SubtitleTrackInfo } from "./media";

export interface PlaybackState {
  movie_id: string;
  media_id: string;
  position_seconds: number;
  duration_seconds: number;
  completed: boolean;
  updated_at: string;
}

export interface PlaybackSession {
  media_type: "movie" | "episode";
  media_id: string;
  movie_id?: string | null;
  series_id?: string | null;
  season_number?: number | null;
  episode_number?: number | null;
  episode_id?: string | null;
  title: string;
  subtitle_info?: string | null;
  media_path: string;
  stream_url: string;
  position_seconds: number;
  duration_seconds: number;
  is_playing: boolean;
  is_fullscreen: boolean;
  volume: number;
  is_muted: boolean;
  playback_speed: number;
  current_audio_track: string | null;
  current_subtitle_track: string | null;
  audio_tracks: AudioTrackInfo[];
  subtitle_tracks: SubtitleTrackInfo[];
  requires_resume_prompt: boolean;
  resume_position_seconds: number;
}

export interface PlaybackPositionPayload {
  movie_id: string;
  media_id: string;
  position_seconds: number;
  duration_seconds: number;
}

export interface PlaybackErrorPayload {
  movie_id: string | null;
  message: string;
}
