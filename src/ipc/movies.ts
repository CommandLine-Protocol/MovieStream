import { invoke } from "@tauri-apps/api/core";
import { Movie, MovieFilter, MovieSort, MovieWithMedia } from "../types";

export async function listMovies(
  filter?: MovieFilter,
  sort?: MovieSort
): Promise<Movie[]> {
  return await invoke<Movie[]>("list_movies", { filter, sort });
}

export async function getMovie(movieId: string): Promise<MovieWithMedia | null> {
  return await invoke<MovieWithMedia | null>("get_movie", { movieId });
}

export async function searchMovies(query: string): Promise<Movie[]> {
  return await invoke<Movie[]>("search_movies", { query });
}

export async function setMetadataMatch(
  movieId: string,
  providerId: string
): Promise<Movie> {
  return await invoke<Movie>("set_metadata_match", { movieId, providerId });
}
