use axum::{
    body::Body,
    extract::Query,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::io::SeekFrom;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tower_http::cors::{Any, CorsLayer};

pub struct MediaStreamServer {
    port: u16,
}

#[derive(Deserialize)]
struct StreamQuery {
    path: String,
}

impl MediaStreamServer {
    pub async fn start() -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/stream", get(handle_stream))
            .route("/subtitles", get(handle_subtitles))
            .layer(cors);

        tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });

        tracing::info!("MediaStreamServer listening on http://127.0.0.1:{}", port);
        Ok(Arc::new(Self { port }))
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

async fn handle_stream(
    Query(query): Query<StreamQuery>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    let path = PathBuf::from(&query.path);
    if !path.exists() || !path.is_file() {
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
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
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
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );

    Ok(response)
}

async fn handle_subtitles(
    Query(query): Query<StreamQuery>,
) -> Result<Response, (StatusCode, String)> {
    let path = PathBuf::from(&query.path);
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "Subtitle file not found".to_string()));
    }

    let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot read subtitle: {}", e),
        )
    })?;

    // If SRT, convert to WebVTT format
    let vtt_content = if path.extension().and_then(|s| s.to_str()).map_or(false, |ext| ext.eq_ignore_ascii_case("srt")) {
        convert_srt_to_vtt(&content)
    } else {
        content
    };

    let mut response = (StatusCode::OK, vtt_content).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/vtt; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
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
