import { invoke } from "@tauri-apps/api/core";
import { PlaybackSession } from "../types";

export async function startPlayback(
  movieId: string,
  mediaId: string
): Promise<PlaybackSession> {
  return await invoke<PlaybackSession>("start_playback", { movieId, mediaId });
}

export async function play(): Promise<void> {
  await invoke("play");
}

export async function pause(): Promise<void> {
  await invoke("pause");
}

export async function stop(): Promise<void> {
  await invoke("stop");
}

export async function seek(positionSeconds: number): Promise<void> {
  await invoke("seek", { positionSeconds });
}

export async function resumeAt(positionSeconds: number): Promise<void> {
  await invoke("resume_at", { positionSeconds });
}

export async function setVolume(level: number): Promise<void> {
  await invoke("set_volume", { level });
}

export async function setMute(muted: boolean): Promise<void> {
  await invoke("set_mute", { muted });
}

export async function setFullscreen(enabled: boolean): Promise<void> {
  await invoke("set_fullscreen", { enabled });
}

export async function setPlaybackSpeed(speed: number): Promise<void> {
  await invoke("set_playback_speed", { speed });
}

export async function selectAudioTrack(trackId: string): Promise<void> {
  await invoke("select_audio_track", { trackId });
}

export async function selectSubtitleTrack(trackId: string | null): Promise<void> {
  await invoke("select_subtitle_track", { trackId });
}

export async function loadExternalSubtitle(path: string): Promise<void> {
  await invoke("load_external_subtitle", { path });
}

export async function getActiveSession(): Promise<PlaybackSession | null> {
  return await invoke<PlaybackSession | null>("get_active_session");
}

export async function recordPosition(positionSeconds: number): Promise<void> {
  await invoke("record_position", { positionSeconds });
}
