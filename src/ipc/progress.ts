import { invoke } from "@tauri-apps/api/core";
import { ContinueWatchingItem, MediaProgress, TvEpisode } from "../types";

export async function getContinueWatching(limit?: number): Promise<ContinueWatchingItem[]> {
  return await invoke<ContinueWatchingItem[]>("get_continue_watching", { limit });
}

export async function getPlaybackProgress(mediaId: string): Promise<MediaProgress | null> {
  return await invoke<MediaProgress | null>("get_playback_progress", { mediaId });
}

export async function markMediaCompleted(mediaId: string): Promise<void> {
  await invoke("mark_media_completed", { mediaId });
}

export async function getNextEpisode(): Promise<TvEpisode | null> {
  return await invoke<TvEpisode | null>("get_next_episode");
}
