use std::path::Path;
use std::sync::Arc;

use crate::adapters::vlc::{probe_media_file, LibVlcApi};
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

pub struct MediaAnalyzer {
    vlc_api: Option<Arc<LibVlcApi>>,
}

impl MediaAnalyzer {
    pub fn new() -> Self {
        Self {
            vlc_api: LibVlcApi::try_load(),
        }
    }

    pub fn analyze(&self, media_path: &str, resolution_hint: Option<&str>) -> AppResult<AnalyzedMediaInfo> {
        let path = Path::new(media_path);

        // 1. Attempt real media probing using libVLC FFI
        if let Some(ref api) = self.vlc_api {
            if let Some(probed) = probe_media_file(api, media_path) {
                let mut subtitle_tracks = probed.subtitle_tracks;
                self.collect_external_subtitles(path, &mut subtitle_tracks);

                let (hint_w, hint_h) = Self::resolution_from_hint(resolution_hint);

                return Ok(AnalyzedMediaInfo {
                    duration_seconds: probed.duration_seconds,
                    container_format: probed.container_format,
                    video_codec: probed.video_codec.or_else(|| Self::guess_codec_from_name(media_path)),
                    resolution_width: probed.resolution_width.or(hint_w),
                    resolution_height: probed.resolution_height.or(hint_h),
                    audio_tracks: if !probed.audio_tracks.is_empty() {
                        probed.audio_tracks
                    } else {
                        Self::fallback_audio_tracks(media_path)
                    },
                    subtitle_tracks,
                });
            }
        }

        // 2. Fallback when libVLC is not available or probing didn't succeed
        let container_format = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase());

        let (resolution_width, resolution_height) = Self::resolution_from_hint(resolution_hint);
        let video_codec = Self::guess_codec_from_name(media_path);
        let audio_tracks = Self::fallback_audio_tracks(media_path);

        let mut subtitle_tracks = Vec::new();
        self.collect_external_subtitles(path, &mut subtitle_tracks);

        Ok(AnalyzedMediaInfo {
            duration_seconds: None,
            container_format,
            video_codec,
            resolution_width,
            resolution_height,
            audio_tracks,
            subtitle_tracks,
        })
    }

    fn resolution_from_hint(hint: Option<&str>) -> (Option<u32>, Option<u32>) {
        match hint {
            Some("2160P") | Some("4K") | Some("UHD") => (Some(3840), Some(2160)),
            Some("1080P") | Some("1080I") => (Some(1920), Some(1080)),
            Some("720P") => (Some(1280), Some(720)),
            Some("576P") => (Some(1024), Some(576)),
            Some("480P") => (Some(854), Some(480)),
            _ => (Some(1920), Some(1080)),
        }
    }

    fn guess_codec_from_name(name: &str) -> Option<String> {
        let lower = name.to_lowercase();
        if lower.contains("x265") || lower.contains("hevc") || lower.contains("h265") {
            Some("HEVC / H.265".to_string())
        } else if lower.contains("av1") {
            Some("AV1".to_string())
        } else if lower.contains("vp9") {
            Some("VP9".to_string())
        } else {
            Some("AVC / H.264".to_string())
        }
    }

    fn fallback_audio_tracks(name: &str) -> Vec<AudioTrackInfo> {
        let mut tracks = vec![AudioTrackInfo {
            id: "1".to_string(),
            name: "Default Audio Track".to_string(),
            language: Some("und".to_string()),
            codec: Some("aac".to_string()),
            channels: Some(2),
        }];

        if name.to_lowercase().contains("5.1")
            || name.to_lowercase().contains("dts")
            || name.to_lowercase().contains("ac3")
        {
            tracks.push(AudioTrackInfo {
                id: "2".to_string(),
                name: "Surround 5.1 Track".to_string(),
                language: Some("und".to_string()),
                codec: Some("ac3".to_string()),
                channels: Some(6),
            });
        }

        tracks
    }

    fn collect_external_subtitles(&self, path: &Path, tracks: &mut Vec<SubtitleTrackInfo>) {
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
                                        if !tracks.iter().any(|t| t.name == name) {
                                            tracks.push(SubtitleTrackInfo {
                                                id: format!("ext-{}", tracks.len() + 1),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_media_analyzer_resolution_hint() {
        assert_eq!(MediaAnalyzer::resolution_from_hint(Some("4K")), (Some(3840), Some(2160)));
        assert_eq!(MediaAnalyzer::resolution_from_hint(Some("1080P")), (Some(1920), Some(1080)));
        assert_eq!(MediaAnalyzer::resolution_from_hint(Some("720P")), (Some(1280), Some(720)));
        assert_eq!(MediaAnalyzer::resolution_from_hint(None), (Some(1920), Some(1080)));
    }

    #[test]
    fn test_media_analyzer_codec_guessing() {
        assert_eq!(MediaAnalyzer::guess_codec_from_name("Movie.2024.x265.mkv"), Some("HEVC / H.265".to_string()));
        assert_eq!(MediaAnalyzer::guess_codec_from_name("Movie.2024.av1.mkv"), Some("AV1".to_string()));
        assert_eq!(MediaAnalyzer::guess_codec_from_name("Movie.2024.h264.mkv"), Some("AVC / H.264".to_string()));
    }

    #[test]
    fn test_media_analyzer_external_subtitles_detection() {
        let dir = tempdir().unwrap();
        let video = dir.path().join("Sample.Movie.2024.mp4");
        File::create(&video).unwrap();

        let srt = dir.path().join("Sample.Movie.2024.en.srt");
        File::create(&srt).unwrap();

        let analyzer = MediaAnalyzer::new();
        let info = analyzer.analyze(&video.to_string_lossy(), Some("1080P")).unwrap();

        assert_eq!(info.resolution_width, Some(1920));
        assert_eq!(info.resolution_height, Some(1080));
        assert_eq!(info.container_format, Some("mp4".to_string()));
        assert!(info.subtitle_tracks.iter().any(|s| s.name == "Sample.Movie.2024.en.srt"));
    }
}
