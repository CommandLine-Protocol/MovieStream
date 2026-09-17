use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use libloading::Library;

use crate::domain::{AudioTrackInfo, SubtitleTrackInfo};

#[repr(C)]
pub struct LibVlcAudioTrack {
    pub i_channels: c_uint,
    pub i_rate: c_uint,
}

#[repr(C)]
pub struct LibVlcVideoTrack {
    pub i_height: c_uint,
    pub i_width: c_uint,
    pub i_sar_num: c_uint,
    pub i_sar_den: c_uint,
    pub i_frame_rate_num: c_uint,
    pub i_frame_rate_den: c_uint,
    pub i_orientation: c_int,
    pub i_projection: c_int,
    pub pose: [c_float; 4],
}

#[repr(C)]
pub struct LibVlcSubtitleTrack {
    pub psz_encoding: *mut c_char,
}

#[repr(C)]
pub union LibVlcTrackUnion {
    pub audio: *mut LibVlcAudioTrack,
    pub video: *mut LibVlcVideoTrack,
    pub subtitle: *mut LibVlcSubtitleTrack,
}

#[repr(C)]
pub struct LibVlcMediaTrack {
    pub i_codec: u32,
    pub i_original_fourcc: u32,
    pub i_id: c_int,
    pub i_type: c_int, // 0 = unknown, 1 = audio, 2 = video, 3 = text/subtitle
    pub i_profile: c_int,
    pub i_level: c_int,
    pub u: LibVlcTrackUnion,
    pub i_bitrate: c_uint,
    pub psz_language: *mut c_char,
    pub psz_description: *mut c_char,
}

#[repr(C)]
pub struct LibVlcTrackDescription {
    pub i_id: c_int,
    pub psz_name: *mut c_char,
    pub p_next: *mut LibVlcTrackDescription,
}

#[derive(Debug, Clone)]
pub struct ProbedMediaDetails {
    pub duration_seconds: Option<u32>,
    pub container_format: Option<String>,
    pub video_codec: Option<String>,
    pub resolution_width: Option<u32>,
    pub resolution_height: Option<u32>,
    pub audio_tracks: Vec<AudioTrackInfo>,
    pub subtitle_tracks: Vec<SubtitleTrackInfo>,
}

/// Dynamically loaded libVLC function pointers.
pub struct LibVlcApi {
    _core_lib: Option<Library>,
    _vlc_lib: Library,

    pub libvlc_new: unsafe extern "C" fn(c_int, *const *const c_char) -> *mut c_void,
    pub libvlc_release: unsafe extern "C" fn(*mut c_void),
    pub libvlc_get_version: unsafe extern "C" fn() -> *const c_char,

    pub libvlc_media_new_path: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
    pub libvlc_media_release: unsafe extern "C" fn(*mut c_void),
    pub libvlc_media_parse_with_options: unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int,
    pub libvlc_media_get_parsed_status: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_media_get_duration: unsafe extern "C" fn(*mut c_void) -> i64,
    pub libvlc_media_tracks_get: unsafe extern "C" fn(*mut c_void, *mut *mut *mut LibVlcMediaTrack) -> c_uint,
    pub libvlc_media_tracks_release: unsafe extern "C" fn(*mut *mut LibVlcMediaTrack, c_uint),

    pub libvlc_media_player_new_from_media: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    pub libvlc_media_player_release: unsafe extern "C" fn(*mut c_void),
    pub libvlc_media_player_play: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_media_player_pause: unsafe extern "C" fn(*mut c_void),
    pub libvlc_media_player_stop: unsafe extern "C" fn(*mut c_void),
    pub libvlc_media_player_is_playing: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_media_player_get_time: unsafe extern "C" fn(*mut c_void) -> i64,
    pub libvlc_media_player_set_time: unsafe extern "C" fn(*mut c_void, i64),
    pub libvlc_media_player_get_length: unsafe extern "C" fn(*mut c_void) -> i64,
    pub libvlc_media_player_set_rate: unsafe extern "C" fn(*mut c_void, c_float) -> c_int,
    pub libvlc_media_player_get_rate: unsafe extern "C" fn(*mut c_void) -> c_float,

    pub libvlc_audio_get_volume: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_audio_set_volume: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    pub libvlc_audio_get_mute: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_audio_set_mute: unsafe extern "C" fn(*mut c_void, c_int),
    pub libvlc_audio_get_track: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_audio_set_track: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    pub libvlc_audio_get_track_description: unsafe extern "C" fn(*mut c_void) -> *mut LibVlcTrackDescription,

    pub libvlc_video_get_spu: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub libvlc_video_set_spu: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    pub libvlc_video_set_subtitle_file: unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int,
    pub libvlc_video_get_spu_description: unsafe extern "C" fn(*mut c_void) -> *mut LibVlcTrackDescription,

    pub libvlc_track_description_list_release: unsafe extern "C" fn(*mut LibVlcTrackDescription),
}

// Safety: Library function pointers are safe to share across threads
unsafe impl Send for LibVlcApi {}
unsafe impl Sync for LibVlcApi {}

impl LibVlcApi {
    pub fn try_load() -> Option<Arc<Self>> {
        static INSTANCE: OnceLock<Option<Arc<LibVlcApi>>> = OnceLock::new();
        INSTANCE.get_or_init(Self::load_internal).clone()
    }

    fn load_internal() -> Option<Arc<Self>> {
        let (vlc_path, core_path, plugin_path) = find_vlc_paths()?;

        if let Some(plugins) = plugin_path {
            std::env::set_var("VLC_PLUGIN_PATH", plugins);
        }

        unsafe {
            let core_lib = core_path.and_then(|p| Library::new(p).ok());
            let vlc_lib = Library::new(vlc_path).ok()?;

            macro_rules! get_sym {
                ($name:ident, $type:ty) => {
                    *vlc_lib.get::<$type>(stringify!($name).as_bytes()).ok()?
                };
            }

            let api = Self {
                libvlc_new: get_sym!(libvlc_new, unsafe extern "C" fn(c_int, *const *const c_char) -> *mut c_void),
                libvlc_release: get_sym!(libvlc_release, unsafe extern "C" fn(*mut c_void)),
                libvlc_get_version: get_sym!(libvlc_get_version, unsafe extern "C" fn() -> *const c_char),

                libvlc_media_new_path: get_sym!(libvlc_media_new_path, unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void),
                libvlc_media_release: get_sym!(libvlc_media_release, unsafe extern "C" fn(*mut c_void)),
                libvlc_media_parse_with_options: get_sym!(libvlc_media_parse_with_options, unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int),
                libvlc_media_get_parsed_status: get_sym!(libvlc_media_get_parsed_status, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_media_get_duration: get_sym!(libvlc_media_get_duration, unsafe extern "C" fn(*mut c_void) -> i64),
                libvlc_media_tracks_get: get_sym!(libvlc_media_tracks_get, unsafe extern "C" fn(*mut c_void, *mut *mut *mut LibVlcMediaTrack) -> c_uint),
                libvlc_media_tracks_release: get_sym!(libvlc_media_tracks_release, unsafe extern "C" fn(*mut *mut LibVlcMediaTrack, c_uint)),

                libvlc_media_player_new_from_media: get_sym!(libvlc_media_player_new_from_media, unsafe extern "C" fn(*mut c_void) -> *mut c_void),
                libvlc_media_player_release: get_sym!(libvlc_media_player_release, unsafe extern "C" fn(*mut c_void)),
                libvlc_media_player_play: get_sym!(libvlc_media_player_play, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_media_player_pause: get_sym!(libvlc_media_player_pause, unsafe extern "C" fn(*mut c_void)),
                libvlc_media_player_stop: get_sym!(libvlc_media_player_stop, unsafe extern "C" fn(*mut c_void)),
                libvlc_media_player_is_playing: get_sym!(libvlc_media_player_is_playing, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_media_player_get_time: get_sym!(libvlc_media_player_get_time, unsafe extern "C" fn(*mut c_void) -> i64),
                libvlc_media_player_set_time: get_sym!(libvlc_media_player_set_time, unsafe extern "C" fn(*mut c_void, i64)),
                libvlc_media_player_get_length: get_sym!(libvlc_media_player_get_length, unsafe extern "C" fn(*mut c_void) -> i64),
                libvlc_media_player_set_rate: get_sym!(libvlc_media_player_set_rate, unsafe extern "C" fn(*mut c_void, c_float) -> c_int),
                libvlc_media_player_get_rate: get_sym!(libvlc_media_player_get_rate, unsafe extern "C" fn(*mut c_void) -> c_float),

                libvlc_audio_get_volume: get_sym!(libvlc_audio_get_volume, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_audio_set_volume: get_sym!(libvlc_audio_set_volume, unsafe extern "C" fn(*mut c_void, c_int) -> c_int),
                libvlc_audio_get_mute: get_sym!(libvlc_audio_get_mute, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_audio_set_mute: get_sym!(libvlc_audio_set_mute, unsafe extern "C" fn(*mut c_void, c_int)),
                libvlc_audio_get_track: get_sym!(libvlc_audio_get_track, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_audio_set_track: get_sym!(libvlc_audio_set_track, unsafe extern "C" fn(*mut c_void, c_int) -> c_int),
                libvlc_audio_get_track_description: get_sym!(libvlc_audio_get_track_description, unsafe extern "C" fn(*mut c_void) -> *mut LibVlcTrackDescription),

                libvlc_video_get_spu: get_sym!(libvlc_video_get_spu, unsafe extern "C" fn(*mut c_void) -> c_int),
                libvlc_video_set_spu: get_sym!(libvlc_video_set_spu, unsafe extern "C" fn(*mut c_void, c_int) -> c_int),
                libvlc_video_set_subtitle_file: get_sym!(libvlc_video_set_subtitle_file, unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int),
                libvlc_video_get_spu_description: get_sym!(libvlc_video_get_spu_description, unsafe extern "C" fn(*mut c_void) -> *mut LibVlcTrackDescription),

                libvlc_track_description_list_release: get_sym!(libvlc_track_description_list_release, unsafe extern "C" fn(*mut LibVlcTrackDescription)),

                _core_lib: core_lib,
                _vlc_lib: vlc_lib,
            };

            tracing::info!(
                "LibVLC successfully loaded from: {}",
                CStr::from_ptr((api.libvlc_get_version)()).to_string_lossy()
            );

            Some(Arc::new(api))
        }
    }

    /// Create a new libvlc instance with quiet, headless arguments
    pub fn create_instance(&self) -> Option<*mut c_void> {
        let args = [
            CString::new("--no-xlib").ok()?,
            CString::new("--quiet").ok()?,
            CString::new("--no-video-title-show").ok()?,
            CString::new("--aout=dummy").ok()?,
            CString::new("--vout=dummy").ok()?,
        ];
        let arg_ptrs: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
        let inst = unsafe { (self.libvlc_new)(arg_ptrs.len() as c_int, arg_ptrs.as_ptr()) };
        if inst.is_null() {
            None
        } else {
            Some(inst)
        }
    }
}

fn find_vlc_paths() -> Option<(PathBuf, Option<PathBuf>, Option<PathBuf>)> {
    if let Ok(p) = std::env::var("VLC_LIB_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some((
                path,
                None,
                std::env::var("VLC_PLUGIN_PATH").ok().map(PathBuf::from),
            ));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let vlc_app = Path::new("/Applications/VLC.app");
        let vlc_lib = vlc_app.join("Contents/MacOS/lib/libvlc.dylib");
        let vlccore_lib = vlc_app.join("Contents/MacOS/lib/libvlccore.dylib");
        let plugins = vlc_app.join("Contents/MacOS/plugins");
        if vlc_lib.exists() && vlccore_lib.exists() {
            return Some((vlc_lib, Some(vlccore_lib), Some(plugins)));
        }

        for base in &["/opt/homebrew/lib", "/usr/local/lib"] {
            let p = Path::new(base).join("libvlc.dylib");
            if p.exists() {
                return Some((p, None, None));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for p in &["libvlc.so.5", "libvlc.so"] {
            let pb = PathBuf::from(p);
            return Some((pb, None, None));
        }
    }

    #[cfg(target_os = "windows")]
    {
        for dir in &[
            r"C:\Program Files\VideoLAN\VLC",
            r"C:\Program Files (x86)\VideoLAN\VLC",
        ] {
            let p = Path::new(dir).join("libvlc.dll");
            if p.exists() {
                return Some((p, None, Some(Path::new(dir).join("plugins"))));
            }
        }
    }

    None
}

/// FourCC to human-readable codec name
pub fn fourcc_to_codec_name(fourcc: u32) -> String {
    let bytes = fourcc.to_le_bytes();
    let fourcc_str = match std::str::from_utf8(&bytes) {
        Ok(s) => s.trim().to_lowercase(),
        Err(_) => format!("0x{:08x}", fourcc),
    };

    match fourcc_str.as_str() {
        "avc1" | "h264" | "x264" => "H.264 / AVC".to_string(),
        "hevc" | "h265" | "x265" | "hev1" => "H.265 / HEVC".to_string(),
        "vp90" | "vp9" => "VP9".to_string(),
        "vp80" | "vp8" => "VP8".to_string(),
        "av01" | "av1" => "AV1".to_string(),
        "mp4v" | "xvid" | "divx" => "MPEG-4 Part 2".to_string(),
        "mpgv" | "mpg1" | "mpg2" => "MPEG-1/2".to_string(),
        "vc-1" | "wvc1" => "VC-1".to_string(),
        "theora" | "theo" => "Theora".to_string(),
        "mp4a" | "aac" => "AAC".to_string(),
        "a52" | "ac-3" | "ac3" => "AC-3 / Dolby Digital".to_string(),
        "eac3" | "ec-3" => "E-AC-3 / Dolby Digital Plus".to_string(),
        "dts" | "dtsb" => "DTS".to_string(),
        "flac" => "FLAC".to_string(),
        "mp3" | "mpga" => "MP3".to_string(),
        "opus" => "Opus".to_string(),
        "vorb" => "Vorbis".to_string(),
        "subt" | "tx3g" => "Timed Text / SubRip".to_string(),
        _ => fourcc_str.to_uppercase(),
    }
}

/// Analyze a media file using libVLC media descriptor parsing.
pub fn probe_media_file(api: &LibVlcApi, path: &str) -> Option<ProbedMediaDetails> {
    let inst = api.create_instance()?;
    let c_path = CString::new(path).ok()?;
    let media = unsafe { (api.libvlc_media_new_path)(inst, c_path.as_ptr()) };
    if media.is_null() {
        unsafe { (api.libvlc_release)(inst) };
        return None;
    }

    // Parse media with local flag (timeout 3000ms)
    unsafe { (api.libvlc_media_parse_with_options)(media, 0x00, 3000) };

    // Wait for parse to complete (up to 2 seconds)
    for _ in 0..40 {
        let status = unsafe { (api.libvlc_media_get_parsed_status)(media) };
        if status == 4 || status == 2 || status == 3 {
            // 4: done, 2: failed, 3: timeout
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let dur_ms = unsafe { (api.libvlc_media_get_duration)(media) };
    let duration_seconds = if dur_ms > 0 {
        Some((dur_ms / 1000) as u32)
    } else {
        None
    };

    let mut track_ptr: *mut *mut LibVlcMediaTrack = std::ptr::null_mut();
    let num_tracks = unsafe { (api.libvlc_media_tracks_get)(media, &mut track_ptr) };

    let mut video_codec = None;
    let mut resolution_width = None;
    let mut resolution_height = None;
    let mut audio_tracks = Vec::new();
    let mut subtitle_tracks = Vec::new();

    if !track_ptr.is_null() && num_tracks > 0 {
        for i in 0..num_tracks as usize {
            let track = unsafe { &**track_ptr.add(i) };
            match track.i_type {
                1 => {
                    // Audio track
                    let channels = unsafe {
                        if !track.u.audio.is_null() {
                            Some((*track.u.audio).i_channels)
                        } else {
                            None
                        }
                    };
                    let lang = if !track.psz_language.is_null() {
                        unsafe { CStr::from_ptr(track.psz_language).to_str().ok().map(|s| s.to_string()) }
                    } else {
                        None
                    };
                    let desc = if !track.psz_description.is_null() {
                        unsafe { CStr::from_ptr(track.psz_description).to_str().ok().map(|s| s.to_string()) }
                    } else {
                        None
                    };
                    let codec_name = fourcc_to_codec_name(track.i_codec);
                    let name = desc.unwrap_or_else(|| {
                        format!("{} ({})", lang.as_deref().unwrap_or("Audio"), codec_name)
                    });

                    audio_tracks.push(AudioTrackInfo {
                        id: track.i_id.to_string(),
                        name,
                        language: lang,
                        codec: Some(codec_name),
                        channels,
                    });
                }
                2 => {
                    // Video track
                    if video_codec.is_none() {
                        video_codec = Some(fourcc_to_codec_name(track.i_codec));
                    }
                    unsafe {
                        if !track.u.video.is_null() {
                            if resolution_width.is_none() && (*track.u.video).i_width > 0 {
                                resolution_width = Some((*track.u.video).i_width);
                            }
                            if resolution_height.is_none() && (*track.u.video).i_height > 0 {
                                resolution_height = Some((*track.u.video).i_height);
                            }
                        }
                    }
                }
                3 => {
                    // Subtitle track
                    let lang = if !track.psz_language.is_null() {
                        unsafe { CStr::from_ptr(track.psz_language).to_str().ok().map(|s| s.to_string()) }
                    } else {
                        None
                    };
                    let desc = if !track.psz_description.is_null() {
                        unsafe { CStr::from_ptr(track.psz_description).to_str().ok().map(|s| s.to_string()) }
                    } else {
                        None
                    };
                    let name = desc.unwrap_or_else(|| {
                        format!("{} (Embedded)", lang.as_deref().unwrap_or("Subtitle"))
                    });

                    subtitle_tracks.push(SubtitleTrackInfo {
                        id: track.i_id.to_string(),
                        name,
                        language: lang,
                        is_external: false,
                        path: None,
                    });
                }
                _ => {}
            }
        }
        unsafe { (api.libvlc_media_tracks_release)(track_ptr, num_tracks) };
    }

    unsafe {
        (api.libvlc_media_release)(media);
        (api.libvlc_release)(inst);
    }

    let container_format = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    Some(ProbedMediaDetails {
        duration_seconds,
        container_format,
        video_codec,
        resolution_width,
        resolution_height,
        audio_tracks,
        subtitle_tracks,
    })
}
