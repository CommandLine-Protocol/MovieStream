import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../types";

export async function getSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return await invoke<AppSettings>("update_settings", { settings });
}
