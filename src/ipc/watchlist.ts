import { invoke } from "@tauri-apps/api/core";
import { Movie } from "../types";

export async function addToWatchlist(movieId: string): Promise<void> {
  await invoke("add_to_watchlist", { movieId });
}

export async function removeFromWatchlist(movieId: string): Promise<void> {
  await invoke("remove_from_watchlist", { movieId });
}

export async function isInWatchlist(movieId: string): Promise<boolean> {
  return await invoke<boolean>("is_in_watchlist", { movieId });
}

export async function listWatchlist(): Promise<Movie[]> {
  return await invoke<Movie[]>("list_watchlist");
}
