export interface TmdbMovieResult {
  id: number;
  title: string;
  original_title?: string | null;
  overview?: string | null;
  release_date?: string | null;
  poster_path?: string | null;
  backdrop_path?: string | null;
  vote_average?: number | null;
  genre_ids?: number[] | null;
}

export interface TmdbMovieDetail {
  id: number;
  title: string;
  original_title?: string | null;
  overview?: string | null;
  release_date?: string | null;
  runtime?: number | null;
  poster_path?: string | null;
  backdrop_path?: string | null;
  vote_average?: number | null;
  genres: string[];
  cast: string[];
  director?: string | null;
}

export interface TmdbTvResult {
  id: number;
  name: string;
  original_name?: string | null;
  overview?: string | null;
  first_air_date?: string | null;
  poster_path?: string | null;
  backdrop_path?: string | null;
  vote_average?: number | null;
}

export interface TmdbSeasonSummary {
  id: number;
  season_number: number;
  name: string;
  overview?: string | null;
  poster_path?: string | null;
  episode_count: number;
  air_date?: string | null;
}

export interface TmdbTvDetail {
  id: number;
  name: string;
  original_name?: string | null;
  overview?: string | null;
  first_air_date?: string | null;
  poster_path?: string | null;
  backdrop_path?: string | null;
  vote_average?: number | null;
  genres: string[];
  number_of_seasons: number;
  number_of_episodes: number;
  seasons: TmdbSeasonSummary[];
  cast: string[];
}

export interface TmdbEpisodeDetail {
  id: number;
  episode_number: number;
  season_number: number;
  name: string;
  overview?: string | null;
  still_path?: string | null;
  air_date?: string | null;
  runtime?: number | null;
  vote_average?: number | null;
}

export interface TmdbTrendingItem {
  id: number;
  media_type: "movie" | "tv";
  title: string;
  overview?: string | null;
  poster_path?: string | null;
  backdrop_path?: string | null;
  release_date?: string | null;
  vote_average?: number | null;
}
