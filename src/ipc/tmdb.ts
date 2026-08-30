import { invoke } from "@tauri-apps/api/core";
import {
  TmdbEpisodeDetail,
  TmdbMovieDetail,
  TmdbMovieResult,
  TmdbSeasonSummary,
  TmdbTrendingItem,
  TmdbTvDetail,
  TmdbTvResult,
} from "../types";

export async function searchTmdbMovies(query: string, year?: number): Promise<TmdbMovieResult[]> {
  return await invoke<TmdbMovieResult[]>("search_tmdb_movies", { query, year });
}

export async function getMovieDetails(movieId: number): Promise<TmdbMovieDetail> {
  return await invoke<TmdbMovieDetail>("get_movie_details", { movieId });
}

export async function searchTv(query: string, year?: number): Promise<TmdbTvResult[]> {
  return await invoke<TmdbTvResult[]>("search_tv", { query, year });
}

export async function getTvDetails(seriesId: number): Promise<TmdbTvDetail> {
  return await invoke<TmdbTvDetail>("get_tv_details", { seriesId });
}

export async function getTvSeasons(seriesId: number): Promise<TmdbSeasonSummary[]> {
  return await invoke<TmdbSeasonSummary[]>("get_tv_seasons", { seriesId });
}

export async function getTvEpisodes(
  seriesId: number,
  seasonNumber: number
): Promise<TmdbEpisodeDetail[]> {
  return await invoke<TmdbEpisodeDetail[]>("get_tv_episodes", { seriesId, seasonNumber });
}

export async function getTrending(mediaType?: "movie" | "tv"): Promise<TmdbTrendingItem[]> {
  return await invoke<TmdbTrendingItem[]>("get_trending", { mediaType });
}

export async function getPopularMovies(): Promise<TmdbMovieResult[]> {
  return await invoke<TmdbMovieResult[]>("get_popular_movies");
}

export async function getPopularTv(): Promise<TmdbTvResult[]> {
  return await invoke<TmdbTvResult[]>("get_popular_tv");
}
