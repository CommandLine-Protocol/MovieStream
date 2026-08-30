import { invoke } from "@tauri-apps/api/core";
import { Movie } from "../types";

export async function getRecentlyWatched(limit?: number): Promise<Movie[]> {
  return await invoke<Movie[]>("recently_watched", { limit });
}
