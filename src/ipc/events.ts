import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  PlaybackErrorPayload,
  PlaybackPositionPayload,
  ScanProgressPayload,
} from "../types";

export const EVENT_SCAN_PROGRESS = "library://scan-progress";
export const EVENT_PLAYBACK_POSITION = "playback://position";
export const EVENT_PLAYBACK_ERROR = "playback://error";

export async function onScanProgress(
  callback: (payload: ScanProgressPayload) => void
): Promise<UnlistenFn> {
  return await listen<ScanProgressPayload>(EVENT_SCAN_PROGRESS, (event) => {
    callback(event.payload);
  });
}

export async function onPlaybackPosition(
  callback: (payload: PlaybackPositionPayload) => void
): Promise<UnlistenFn> {
  return await listen<PlaybackPositionPayload>(EVENT_PLAYBACK_POSITION, (event) => {
    callback(event.payload);
  });
}

export async function onPlaybackError(
  callback: (payload: PlaybackErrorPayload) => void
): Promise<UnlistenFn> {
  return await listen<PlaybackErrorPayload>(EVENT_PLAYBACK_ERROR, (event) => {
    callback(event.payload);
  });
}
