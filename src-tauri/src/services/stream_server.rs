use axum::{
    body::Body,
    extract::{Query, State as AxumState},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::io::SeekFrom;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;

/// Token-bucket/window-based rate limiter for incoming stream server requests.
pub struct StreamRateLimiter {
    requests_this_sec: AtomicUsize,
    window_start: Mutex<Instant>,
    max_requests_per_sec: usize,
}

impl StreamRateLimiter {
    pub fn new(max_requests_per_sec: usize) -> Self {
        Self {
            requests_this_sec: AtomicUsize::new(0),
            window_start: Mutex::new(Instant::now()),
            max_requests_per_sec,
        }
    }

    pub fn check(&self) -> bool {
        let mut start = self.window_start.lock().unwrap();
        let now = Instant::now();
        if now.duration_since(*start).as_secs() >= 1 {
            *start = now;
            self.requests_this_sec.store(1, Ordering::Relaxed);
            true
        } else {
            let count = self.requests_this_sec.fetch_add(1, Ordering::Relaxed);
            count < self.max_requests_per_sec
        }
    }
}

/// Shared set of canonical directory paths that the stream server is allowed to serve files from.
/// Populated with registered library sources and the artwork cache directory.
type AllowedPaths = Arc<RwLock<HashSet<PathBuf>>>;

pub struct MediaStreamServer {
    port: u16,
    allowed_paths: AllowedPaths,
}

#[derive(Deserialize)]
struct StreamQuery {
    path: String,
}

impl MediaStreamServer {
    pub async fn start(
        artwork_cache_dir: PathBuf,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        // Initialize allowed paths with the artwork cache directory
        let mut initial_paths = HashSet::new();
        if let Ok(canonical) = std::fs::canonicalize(&artwork_cache_dir) {
            initial_paths.insert(canonical);
        } else {
            initial_paths.insert(artwork_cache_dir);
        }
        let allowed_paths: AllowedPaths = Arc::new(RwLock::new(initial_paths));
        let allowed_for_router = allowed_paths.clone();

        // Only allow requests from Tauri webview origins and local dev servers
        let cors = CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::predicate(
                |origin: &HeaderValue, _| {
                    let s = origin.to_str().unwrap_or("");
                    s == "tauri://localhost"
                        || s == "https://tauri.localhost"
                        || s.starts_with("http://localhost:")
                        || s.starts_with("http://127.0.0.1:")
                        || s == "http://localhost"
                        || s == "http://127.0.0.1"
                },
            ))
            .allow_methods([axum::http::Method::GET, axum::http::Method::HEAD, axum::http::Method::OPTIONS])
            .allow_headers(tower_http::cors::Any);

        let rate_limiter = Arc::new(StreamRateLimiter::new(100));
        let limiter_for_mw = rate_limiter.clone();

        let app = Router::new()
            .route("/stream", get(handle_stream))
            .route("/subtitles", get(handle_subtitles))
            .with_state(allowed_for_router)
            .layer(cors)
            .layer(tower::limit::ConcurrencyLimitLayer::new(64))
            .layer(axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let limiter = limiter_for_mw.clone();
                    async move {
                        if !limiter.check() {
                            return Err((
                                StatusCode::TOO_MANY_REQUESTS,
                                "Rate limit exceeded".to_string(),
                            ));
                        }
                        Ok::<_, (StatusCode, String)>(next.run(req).await)
                    }
                },
            ));

        tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });

        tracing::info!("MediaStreamServer listening on http://127.0.0.1:{}", port);
        Ok(Arc::new(Self { port, allowed_paths }))
    }

    /// Register a directory as an allowed base path for serving files.
    /// Call this when a new library source is added.
    pub fn register_allowed_path(&self, dir: &Path) {
        if let Ok(canonical) = std::fs::canonicalize(dir) {
            if let Ok(mut paths) = self.allowed_paths.write() {
                paths.insert(canonical);
            }
        } else if let Ok(mut paths) = self.allowed_paths.write() {
            paths.insert(dir.to_path_buf());
        }
    }

    /// Remove a directory from the allowed paths.
    /// Call this when a library source is removed.
    pub fn unregister_allowed_path(&self, dir: &Path) {
        if let Ok(canonical) = std::fs::canonicalize(dir) {
            if let Ok(mut paths) = self.allowed_paths.write() {
                paths.remove(&canonical);
            }
        }
        if let Ok(mut paths) = self.allowed_paths.write() {
            paths.remove(&dir.to_path_buf());
        }
    }

    pub fn get_stream_url(&self, file_path: &str) -> String {
        format!(
            "http://127.0.0.1:{}/stream?path={}",
            self.port,
            urlencoding::encode(file_path)
        )
    }

    pub fn get_subtitle_url(&self, subtitle_path: &str) -> String {
        format!(
            "http://127.0.0.1:{}/subtitles?path={}",
            self.port,
            urlencoding::encode(subtitle_path)
        )
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

/// Validate that the given path is a descendant of one of the allowed directories.
/// Returns the canonical path if valid, or an error.
fn validate_path(
    requested_path: &Path,
    allowed_paths: &AllowedPaths,
) -> Result<PathBuf, (StatusCode, String)> {
    // Canonicalize to resolve symlinks, "..", and "." components
    let canonical = std::fs::canonicalize(requested_path).map_err(|_| {
        (StatusCode::NOT_FOUND, "File not found".to_string())
    })?;

    let paths = allowed_paths.read().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;

    for allowed_dir in paths.iter() {
        if canonical.starts_with(allowed_dir) {
            return Ok(canonical);
        }
    }

    tracing::warn!(
        "Path traversal blocked: requested '{}' (canonical: '{}') is not within any allowed directory",
        requested_path.display(),
        canonical.display()
    );
    Err((StatusCode::FORBIDDEN, "Access denied".to_string()))
}

async fn handle_stream(
    AxumState(allowed_paths): AxumState<AllowedPaths>,
    Query(query): Query<StreamQuery>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    let requested_path = PathBuf::from(&query.path);
    let path = validate_path(&requested_path, &allowed_paths)?;

    if !path.is_file() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    let mut file = File::open(&path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot open file: {}", e),
        )
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot read metadata: {}", e),
        )
    })?;

    let file_size = metadata.len();
    let mime_type = detect_mime_type(&path);

    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    if let Some(range_str) = range_header {
        if let Some(range) = parse_range(range_str, file_size) {
            let (start, end) = range;
            let length = end - start + 1;

            file.seek(SeekFrom::Start(start)).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Seek failed: {}", e),
                )
            })?;

            let stream = ReaderStream::with_capacity(file.take(length), 64 * 1024);
            let body = Body::from_stream(stream);

            let mut response = (StatusCode::PARTIAL_CONTENT, body).into_response();
            let headers = response.headers_mut();

            headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type).unwrap_or(HeaderValue::from_static("video/mp4")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&length.to_string()).unwrap(),
            );
            headers.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, file_size)).unwrap(),
            );

            return Ok(response);
        }
    }

    // No range requested -> stream entire file
    let stream = ReaderStream::with_capacity(file, 64 * 1024);
    let body = Body::from_stream(stream);

    let mut response = (StatusCode::OK, body).into_response();
    let headers = response.headers_mut();

    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type).unwrap_or(HeaderValue::from_static("video/mp4")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&file_size.to_string()).unwrap(),
    );

    Ok(response)
}

async fn handle_subtitles(
    AxumState(allowed_paths): AxumState<AllowedPaths>,
    Query(query): Query<StreamQuery>,
) -> Result<Response, (StatusCode, String)> {
    let requested_path = PathBuf::from(&query.path);
    let path = validate_path(&requested_path, &allowed_paths)?;

    // Verify the file has a subtitle extension
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !matches!(ext.to_lowercase().as_str(), "srt" | "vtt" | "ass" | "ssa" | "sub") {
        return Err((StatusCode::FORBIDDEN, "Not a subtitle file".to_string()));
    }

    // Verify file size does not exceed 10MB to prevent memory exhaustion
    const MAX_SUBTITLE_SIZE: u64 = 10 * 1024 * 1024;
    let metadata = tokio::fs::metadata(&path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot read subtitle metadata: {}", e),
        )
    })?;

    if metadata.len() > MAX_SUBTITLE_SIZE {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Subtitle file exceeds maximum allowed size of 10MB ({} bytes)",
                metadata.len()
            ),
        ));
    }

    let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot read subtitle: {}", e),
        )
    })?;

    // If SRT, convert to WebVTT format
    let vtt_content = if ext.eq_ignore_ascii_case("srt") {
        convert_srt_to_vtt(&content)
    } else {
        content
    };

    let mut response = (StatusCode::OK, vtt_content).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/vtt; charset=utf-8"),
    );

    Ok(response)
}

fn detect_mime_type(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref() {
        Some("mp4") | Some("m4v") => "video/mp4".to_string(),
        Some("mkv") => "video/mp4".to_string(), // Served as video stream for browser demuxer
        Some("webm") => "video/webm".to_string(),
        Some("mov") => "video/quicktime".to_string(),
        Some("avi") => "video/x-msvideo".to_string(),
        Some("ts") | Some("m2ts") => "video/mp2t".to_string(),
        Some("flv") => "video/x-flv".to_string(),
        Some("mp3") => "audio/mpeg".to_string(),
        Some("flac") => "audio/flac".to_string(),
        Some("aac") => "audio/aac".to_string(),
        Some("wav") => "audio/wav".to_string(),
        Some("ogg") | Some("ogv") => "video/ogg".to_string(),
        _ => "video/mp4".to_string(),
    }
}

fn parse_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range_header.starts_with("bytes=") {
        return None;
    }

    let range = &range_header[6..];
    let parts: Vec<&str> = range.split('-').collect();

    match parts.len() {
        2 => {
            let start_str = parts[0];
            let end_str = parts[1];

            if start_str.is_empty() {
                // Suffix byte range: e.g. -500
                let suffix_len: u64 = end_str.parse().ok()?;
                let start = file_size.saturating_sub(suffix_len);
                let end = file_size.saturating_sub(1);
                Some((start, end))
            } else if end_str.is_empty() {
                // Prefix range: e.g. 500-
                let start: u64 = start_str.parse().ok()?;
                if start >= file_size {
                    return None;
                }
                let end = file_size.saturating_sub(1);
                Some((start, end))
            } else {
                // Full range: e.g. 500-999
                let start: u64 = start_str.parse().ok()?;
                let end: u64 = end_str.parse().ok()?;
                if start > end || start >= file_size {
                    return None;
                }
                let end = end.min(file_size.saturating_sub(1));
                Some((start, end))
            }
        }
        _ => None,
    }
}

fn convert_srt_to_vtt(srt: &str) -> String {
    let mut vtt = String::from("WEBVTT\n\n");
    for line in srt.lines() {
        // Convert timestamp separator from comma to dot (e.g. 00:01:20,000 -> 00:01:20.000)
        if line.contains("-->") {
            vtt.push_str(&line.replace(',', "."));
        } else {
            vtt.push_str(line);
        }
        vtt.push('\n');
    }
    vtt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_rate_limiter() {
        let limiter = StreamRateLimiter::new(3);
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(limiter.check());
        // 4th request exceeds rate limit of 3 req/sec
        assert!(!limiter.check());
    }
}
