import { invoke } from "@tauri-apps/api/core";
import { PlaybackSession, SeriesDetails, TvSeries } from "../types";

export async function listTvSeries(): Promise<TvSeries[]> {
  return await invoke<TvSeries[]>("list_tv_series");
}

export async function getSeriesDetails(seriesId: string): Promise<SeriesDetails | null> {
  return await invoke<SeriesDetails | null>("get_series_details", { seriesId });
}

export async function startEpisodePlayback(
  episodeId: string,
  mediaId: string
): Promise<PlaybackSession> {
  return await invoke<PlaybackSession>("start_episode_playback", { episodeId, mediaId });
}
