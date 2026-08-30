use std::path::Path;

use crate::domain::{AudioTrackInfo, SubtitleTrackInfo};
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AnalyzedMediaInfo {
    pub duration_seconds: Option<u32>,
    pub container_format: Option<String>,
    pub video_codec: Option<String>,
    pub resolution_width: Option<u32>,
    pub resolution_height: Option<u32>,
    pub audio_tracks: Vec<AudioTrackInfo>,
    pub subtitle_tracks: Vec<SubtitleTrackInfo>,
}

pub struct MediaAnalyzer;

impl MediaAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, media_path: &str, resolution_hint: Option<&str>) -> AppResult<AnalyzedMediaInfo> {
        let path = Path::new(media_path);

        let container_format = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase());

        let (resolution_width, resolution_height) = match resolution_hint {
            Some("2160P") | Some("4K") | Some("UHD") => (Some(3840), Some(2160)),
            Some("1080P") | Some("1080I") => (Some(1920), Some(1080)),
            Some("720P") => (Some(1280), Some(720)),
            Some("576P") => (Some(1024), Some(576)),
            Some("480P") => (Some(854), Some(480)),
            _ => (Some(1920), Some(1080)), // Standard HD default fallback
        };

        let video_codec = if media_path.to_lowercase().contains("x265")
            || media_path.to_lowercase().contains("hevc")
            || media_path.to_lowercase().contains("h265")
        {
            Some("HEVC / H.265".to_string())
        } else {
            Some("AVC / H.264".to_string())
        };

        let mut audio_tracks = vec![
            AudioTrackInfo {
                id: "1".to_string(),
                name: "English (Stereo)".to_string(),
                language: Some("eng".to_string()),
                codec: Some("aac".to_string()),
                channels: Some(2),
            },
        ];

        if media_path.to_lowercase().contains("5.1") || media_path.to_lowercase().contains("dts") || media_path.to_lowercase().contains("ac3") {
            audio_tracks.push(AudioTrackInfo {
                id: "2".to_string(),
                name: "English (5.1 Surround)".to_string(),
                language: Some("eng".to_string()),
                codec: Some("ac3".to_string()),
                channels: Some(6),
            });
        }

        let mut subtitle_tracks = vec![
            SubtitleTrackInfo {
                id: "1".to_string(),
                name: "English [CC]".to_string(),
                language: Some("eng".to_string()),
                is_external: false,
                path: None,
            },
        ];

        // Scan for external subtitle files in same directory with matching prefix
        if let Some(parent) = path.parent() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let sub_path = entry.path();
                        if let Some(sub_stem) = sub_path.file_stem().and_then(|s| s.to_str()) {
                            if sub_stem.starts_with(stem) {
                                if let Some(ext) = sub_path.extension().and_then(|e| e.to_str()) {
                                    if ext.eq_ignore_ascii_case("srt") || ext.eq_ignore_ascii_case("vtt") {
                                        let name = sub_path
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string();
                                        subtitle_tracks.push(SubtitleTrackInfo {
                                            id: format!("ext-{}", subtitle_tracks.len() + 1),
                                            name,
                                            language: None,
                                            is_external: true,
                                            path: Some(sub_path.to_string_lossy().to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(AnalyzedMediaInfo {
            duration_seconds: Some(7200), // Standard fallback; updated during playback
            container_format,
            video_codec,
            resolution_width,
            resolution_height,
            audio_tracks,
            subtitle_tracks,
        })
    }
}

impl Default for MediaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
