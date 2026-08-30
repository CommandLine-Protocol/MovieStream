export type MediaAvailability = "available" | "unavailable";

export interface AudioTrackInfo {
  id: string;
  name: string;
  language: string | null;
  codec: string | null;
  channels: number | null;
}

export interface SubtitleTrackInfo {
  id: string;
  name: string;
  language: string | null;
  is_external: boolean;
  path: string | null;
}

export interface Media {
  id: string;
  movie_id: string;
  source_id: string;
  path: string;
  size_bytes: number;
  duration_seconds: number | null;
  container_format: string | null;
  video_codec: string | null;
  resolution_width: number | null;
  resolution_height: number | null;
  audio_tracks: AudioTrackInfo[];
  subtitle_tracks: SubtitleTrackInfo[];
  file_hash: string | null;
  file_mtime: string;
  availability: MediaAvailability;
  created_at: string;
  updated_at: string;
}
