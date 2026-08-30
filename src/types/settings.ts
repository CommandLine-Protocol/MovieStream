export interface LibrarySettings {
  scan_on_startup: boolean;
  default_locations: string[];
}

export interface PlaybackSettings {
  default_volume: number;
  default_speed: number;
  resume_behavior: "prompt" | "always" | "never";
  completion_threshold: number;
  audio_language_preference: string | null;
  subtitle_language_preference: string | null;
  subtitles_enabled_by_default: boolean;
}

export interface AppearanceSettings {
  theme: "dark" | "cinematic" | "midnight";
  animations_enabled: boolean;
}

export interface MetadataSettings {
  active_provider_id: string;
  artwork_caching_enabled: boolean;
  auto_match_threshold: number;
}

export interface ApplicationSettings {
  launch_on_startup: boolean;
  notifications_enabled: boolean;
  log_level: string;
}

export interface AppSettings {
  library: LibrarySettings;
  playback: PlaybackSettings;
  appearance: AppearanceSettings;
  metadata: MetadataSettings;
  application: ApplicationSettings;
}
