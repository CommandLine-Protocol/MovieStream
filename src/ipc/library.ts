import { invoke } from "@tauri-apps/api/core";
import { LibrarySource } from "../types";

export async function addSource(path: string): Promise<LibrarySource> {
  return await invoke<LibrarySource>("add_source", { path });
}

export async function pickAndAddSource(): Promise<LibrarySource | null> {
  return await invoke<LibrarySource | null>("pick_and_add_source");
}

export async function removeSource(sourceId: string): Promise<void> {
  await invoke("remove_source", { sourceId });
}

export async function listSources(): Promise<LibrarySource[]> {
  return await invoke<LibrarySource[]>("list_sources");
}

export async function rescanSource(sourceId: string): Promise<void> {
  await invoke("rescan_source", { sourceId });
}

export async function rescanAll(): Promise<void> {
  await invoke("rescan_all");
}
