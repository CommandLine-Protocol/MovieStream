export type MediaType = "movie" | "episode";

export interface MediaProgress {
  id: string;
  media_type: MediaType;
  media_id: string;
  movie_id?: string | null;
  series_id?: string | null;
  season_number?: number | null;
  episode_number?: number | null;
  episode_id?: string | null;
  position_seconds: number;
  duration_seconds: number;
  progress_percentage: number;
  completed: boolean;
  last_watched: string;
}

export interface ContinueWatchingItem {
  progress: MediaProgress;
  movie_title?: string | null;
  movie_poster?: string | null;
  movie_backdrop?: string | null;
  movie_year?: number | null;
  series_title?: string | null;
  series_poster?: string | null;
  episode_title?: string | null;
  episode_still?: string | null;
}
