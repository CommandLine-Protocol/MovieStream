export interface TvSeries {
  id: string;
  tmdb_id?: number | null;
  title: string;
  original_title?: string | null;
  year?: number | null;
  description?: string | null;
  poster_path?: string | null;
  backdrop_path?: string | null;
  genres: string[];
  rating?: number | null;
  created_at: string;
  updated_at: string;
}

export interface TvSeason {
  id: string;
  series_id: string;
  season_number: number;
  name: string;
  overview?: string | null;
  poster_path?: string | null;
  episode_count: number;
  created_at: string;
}

export interface TvEpisode {
  id: string;
  series_id: string;
  season_id: string;
  season_number: number;
  episode_number: number;
  title: string;
  overview?: string | null;
  still_path?: string | null;
  air_date?: string | null;
  duration_seconds?: number | null;
  rating?: number | null;
  created_at: string;
  updated_at: string;
}

export interface EpisodeWithMedia {
  episode: TvEpisode;
  media_path?: string | null;
  media_id?: string | null;
  progress_seconds: number;
  duration_seconds: number;
  completed: boolean;
}

export interface SeasonWithEpisodes {
  season: TvSeason;
  episodes: EpisodeWithMedia[];
}

export interface SeriesDetails {
  series: TvSeries;
  seasons: SeasonWithEpisodes[];
  total_episodes: number;
}
