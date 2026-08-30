use moviestream_lib::services::MediaStreamServer;
use tempfile::tempdir;

#[tokio::test]
async fn test_media_stream_server_range_requests() {
    let tmp = tempdir().unwrap();
    let sample_video = tmp.path().join("sample.mp4");
    // Write 10KB of test byte data
    let test_data = vec![0x42u8; 10240];
    tokio::fs::write(&sample_video, &test_data).await.unwrap();

    let server = MediaStreamServer::start().await.expect("Server starts");
    let stream_url = server.get_stream_url(&sample_video.to_string_lossy());

    let client = reqwest::Client::new();

    // 1. Test standard GET request
    let resp = client.get(&stream_url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.headers().get("accept-ranges").unwrap(), "bytes");
    let bytes = resp.bytes().await.unwrap();
    assert_eq!(bytes.len(), 10240);

    // 2. Test Range request: bytes=0-499 (500 bytes)
    let range_resp = client
        .get(&stream_url)
        .header("Range", "bytes=0-499")
        .send()
        .await
        .unwrap();
    assert_eq!(range_resp.status(), reqwest::StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        range_resp.headers().get("content-range").unwrap(),
        "bytes 0-499/10240"
    );
    let range_bytes = range_resp.bytes().await.unwrap();
    assert_eq!(range_bytes.len(), 500);

    // 3. Test Range request: bytes=5000- (from 5000 to end)
    let range_resp2 = client
        .get(&stream_url)
        .header("Range", "bytes=5000-")
        .send()
        .await
        .unwrap();
    assert_eq!(range_resp2.status(), reqwest::StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        range_resp2.headers().get("content-range").unwrap(),
        "bytes 5000-10239/10240"
    );
    let range_bytes2 = range_resp2.bytes().await.unwrap();
    assert_eq!(range_bytes2.len(), 5240);
}

#[tokio::test]
async fn test_media_stream_server_subtitle_conversion() {
    let tmp = tempdir().unwrap();
    let srt_file = tmp.path().join("sample.srt");
    let srt_content = "1\n00:00:01,000 --> 00:00:04,000\nHello World\n";
    tokio::fs::write(&srt_file, srt_content).await.unwrap();

    let server = MediaStreamServer::start().await.expect("Server starts");
    let subtitle_url = server.get_subtitle_url(&srt_file.to_string_lossy());

    let client = reqwest::Client::new();
    let resp = client.get(&subtitle_url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/vtt; charset=utf-8"
    );

    let vtt_text = resp.text().await.unwrap();
    assert!(vtt_text.starts_with("WEBVTT"));
    assert!(vtt_text.contains("00:00:01.000 --> 00:00:04.000"));
}
