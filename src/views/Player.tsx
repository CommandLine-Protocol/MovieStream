import React, { useState, useEffect, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { usePlayback } from "../state/PlaybackContext";
import { useToast } from "../state/ToastContext";
import {
  Play,
  Pause,
  RotateCcw,
  RotateCw,
  Volume2,
  VolumeX,
  Volume1,
  Maximize,
  Minimize,
  Subtitles,
  AudioLines,
  X,
  Gauge,
  Camera,
  Crop,
  StepForward,
  StepBack,
  Sliders,
  PictureInPicture,
  SkipForward,
} from "lucide-react";

type AspectRatioMode = "fit" | "16:9" | "21:9" | "4:3" | "fill";

export const Player: React.FC = () => {
  const {
    session,
    activeMovie,
    isPlayerOpen,
    nextEpisode,
    closePlayer,
    togglePlay,
    seek,
    setVolume,
    toggleMute,
    setSpeed,
    selectAudio,
    selectSubtitle,
    playNextEpisode,
  } = usePlayback();

  const { showToast } = useToast();

  const [showControls, setShowControls] = useState(true);
  const [showAudioMenu, setShowAudioMenu] = useState(false);
  const [showSubtitleMenu, setShowSubtitleMenu] = useState(false);
  const [showSpeedMenu, setShowSpeedMenu] = useState(false);
  const [showAspectMenu, setShowAspectMenu] = useState(false);
  const [showSyncMenu, setShowSyncMenu] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [aspectRatio, setAspectRatio] = useState<AspectRatioMode>("fit");
  const [subtitleOffsetMs, setSubtitleOffsetMs] = useState(0);
  const [customVolume, setCustomVolume] = useState(80); // 0 to 200%

  const containerRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const hideTimeoutRef = useRef<number | null>(null);

  // Auto-hide controls overlay after 3.5 seconds of mouse inactivity
  useEffect(() => {
    const handleMouseMove = () => {
      setShowControls(true);
      if (hideTimeoutRef.current) clearTimeout(hideTimeoutRef.current);
      hideTimeoutRef.current = window.setTimeout(() => {
        if (session?.is_playing) {
          setShowControls(false);
          setShowAudioMenu(false);
          setShowSubtitleMenu(false);
          setShowSpeedMenu(false);
          setShowAspectMenu(false);
          setShowSyncMenu(false);
        }
      }, 3500);
    };

    window.addEventListener("mousemove", handleMouseMove);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      if (hideTimeoutRef.current) clearTimeout(hideTimeoutRef.current);
    };
  }, [session?.is_playing]);

  // Audio Booster (GainNode for volume up to 200%)
  useEffect(() => {
    if (!videoRef.current) return;

    if (!audioContextRef.current && customVolume > 100) {
      try {
        const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
        const source = ctx.createMediaElementSource(videoRef.current);
        const gain = ctx.createGain();
        source.connect(gain);
        gain.connect(ctx.destination);
        audioContextRef.current = ctx;
        gainNodeRef.current = gain;
      } catch (err) {
        console.warn("AudioContext setup failed:", err);
      }
    }

    if (gainNodeRef.current) {
      gainNodeRef.current.gain.value = customVolume / 100;
    }
  }, [customVolume]);

  // Sync internal HTML5 video element with playback session changes
  useEffect(() => {
    if (!videoRef.current || !session) return;

    if (session.is_playing && videoRef.current.paused) {
      videoRef.current.play().catch(console.error);
    } else if (!session.is_playing && !videoRef.current.paused) {
      videoRef.current.pause();
    }

    if (Math.abs(videoRef.current.currentTime - session.position_seconds) > 2) {
      videoRef.current.currentTime = session.position_seconds;
    }

    videoRef.current.volume = Math.min(1, Math.max(0, customVolume / 100));
    videoRef.current.muted = session.is_muted;
  }, [session?.is_playing, session?.position_seconds, session?.is_muted, customVolume]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isPlayerOpen || !session) return;

      switch (e.key) {
        case " ":
        case "k":
          e.preventDefault();
          togglePlay();
          break;
        case "ArrowLeft":
        case "j":
          e.preventDefault();
          const prev = Math.max(0, session.position_seconds - 10);
          if (videoRef.current) videoRef.current.currentTime = prev;
          seek(prev);
          break;
        case "ArrowRight":
        case "l":
          e.preventDefault();
          const next = session.position_seconds + 10;
          if (videoRef.current) videoRef.current.currentTime = next;
          seek(next);
          break;
        case "ArrowUp":
          e.preventDefault();
          setCustomVolume((v) => {
            const nv = Math.min(200, v + 10);
            setVolume(Math.min(100, nv));
            return nv;
          });
          break;
        case "ArrowDown":
          e.preventDefault();
          setCustomVolume((v) => {
            const nv = Math.max(0, v - 10);
            setVolume(Math.min(100, nv));
            return nv;
          });
          break;
        case "f":
          e.preventDefault();
          toggleFullscreen();
          break;
        case "m":
          e.preventDefault();
          toggleMute();
          break;
        case "Escape":
          e.preventDefault();
          if (isFullscreen) {
            document.exitFullscreen().catch(console.error);
            setIsFullscreen(false);
          } else {
            closePlayer();
          }
          break;
        case "s":
          e.preventDefault();
          captureSnapshot();
          break;
        case ",":
          e.preventDefault();
          stepFrame(false);
          break;
        case ".":
          e.preventDefault();
          stepFrame(true);
          break;
        default:
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isPlayerOpen, session, isFullscreen, togglePlay, seek, setVolume, toggleMute, closePlayer]);

  if (!isPlayerOpen || !session) return null;

  const title = session.title || activeMovie?.title || "Video";
  const subtitle = session.subtitle_info || (activeMovie?.year ? `${activeMovie.year}` : "");

  const formatTime = (seconds: number) => {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const progressPercent = session.duration_seconds > 0
    ? (session.position_seconds / session.duration_seconds) * 100
    : 0;

  const handleTimelineClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const pos = (e.clientX - rect.left) / rect.width;
    const targetSeconds = Math.floor(pos * session.duration_seconds);
    if (videoRef.current) videoRef.current.currentTime = targetSeconds;
    seek(targetSeconds);
  };

  const toggleFullscreen = () => {
    if (!containerRef.current) return;
    if (!document.fullscreenElement) {
      containerRef.current.requestFullscreen().then(() => setIsFullscreen(true)).catch(console.error);
    } else {
      document.exitFullscreen().then(() => setIsFullscreen(false)).catch(console.error);
    }
  };

  const togglePiP = async () => {
    if (!videoRef.current) return;
    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture();
      } else {
        await videoRef.current.requestPictureInPicture();
      }
    } catch (err) {
      showToast({ title: "PiP Error", message: "Picture in picture not supported on this video format", type: "error" });
    }
  };

  const stepFrame = (forward: boolean) => {
    if (!videoRef.current) return;
    const frameTime = 1 / 24;
    videoRef.current.pause();
    const newTime = forward
      ? videoRef.current.currentTime + frameTime
      : Math.max(0, videoRef.current.currentTime - frameTime);
    videoRef.current.currentTime = newTime;
    seek(Math.floor(newTime));
  };

  const captureSnapshot = () => {
    if (!videoRef.current) return;
    try {
      const canvas = document.createElement("canvas");
      canvas.width = videoRef.current.videoWidth || 1920;
      canvas.height = videoRef.current.videoHeight || 1080;
      const ctx = canvas.getContext("2d");
      if (ctx) {
        ctx.drawImage(videoRef.current, 0, 0, canvas.width, canvas.height);
        const dataUrl = canvas.toDataURL("image/png");
        const a = document.createElement("a");
        a.href = dataUrl;
        a.download = `${title.replace(/\s+/g, "_")}_snapshot_${session.position_seconds}s.png`;
        a.click();
        showToast({ title: "Snapshot Saved", message: "Frame captured to downloads", type: "success" });
      }
    } catch (err) {
      showToast({ title: "Snapshot Error", message: "Could not capture frame from stream", type: "error" });
    }
  };

  const getVideoStyle = (): React.CSSProperties => {
    switch (aspectRatio) {
      case "16:9":
        return { width: "100%", aspectRatio: "16/9", objectFit: "contain" };
      case "21:9":
        return { width: "100%", aspectRatio: "21/9", objectFit: "contain" };
      case "4:3":
        return { width: "100%", aspectRatio: "4/3", objectFit: "contain" };
      case "fill":
        return { width: "100%", height: "100%", objectFit: "fill" };
      case "fit":
      default:
        return { width: "100%", height: "100%", objectFit: "contain" };
    }
  };

  const streamSrc = session.stream_url.startsWith("http")
    ? session.stream_url
    : convertFileSrc(session.media_path);

  const selectedSubtitleTrack = session.subtitle_tracks.find((t) => t.id === session.current_subtitle_track);

  return (
    <div
      ref={containerRef}
      className="player-container"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 999,
        background: "#000",
        userSelect: "none",
        cursor: showControls ? "default" : "none",
      }}
    >
      {/* Native In-Window Video Player Element */}
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "#000",
          position: "relative",
          overflow: "hidden",
        }}
        onClick={togglePlay}
      >
        <video
          ref={videoRef}
          src={streamSrc}
          autoPlay
          playsInline
          crossOrigin="anonymous"
          style={getVideoStyle()}
          onTimeUpdate={(e) => {
            const cur = Math.floor(e.currentTarget.currentTime);
            if (cur !== session.position_seconds) {
              seek(cur);
            }
          }}
          onLoadedMetadata={(e) => {
            const dur = Math.floor(e.currentTarget.duration);
            if (dur > 0 && session.position_seconds > 0) {
              e.currentTarget.currentTime = session.position_seconds;
            }
          }}
          onPlay={() => {
            if (!session.is_playing) togglePlay();
          }}
          onPause={() => {
            if (session.is_playing) togglePlay();
          }}
        >
          {/* Subtitle track */}
          {selectedSubtitleTrack?.path && (
            <track
              kind="subtitles"
              label={selectedSubtitleTrack.name}
              src={selectedSubtitleTrack.path}
              default
            />
          )}
        </video>
      </div>

      {/* Floating Next Episode Notification (if available and near end) */}
      {nextEpisode && (
        <div
          className="glass-panel"
          style={{
            position: "absolute",
            bottom: showControls ? 110 : 30,
            right: 32,
            padding: "16px 20px",
            display: "flex",
            alignItems: "center",
            gap: 16,
            zIndex: 1000,
            boxShadow: "0 12px 36px rgba(0, 0, 0, 0.8)",
            border: "1px solid rgba(229, 9, 20, 0.4)",
            animation: "fadeIn 0.3s ease",
          }}
        >
          <div>
            <div style={{ fontSize: "0.75rem", textTransform: "uppercase", color: "var(--accent-primary)", fontWeight: 700 }}>
              Up Next
            </div>
            <div style={{ fontSize: "0.95rem", fontWeight: 700 }}>
              S{nextEpisode.season_number.toString().padStart(2, "0")}E{nextEpisode.episode_number.toString().padStart(2, "0")} • {nextEpisode.title}
            </div>
          </div>
          <button className="btn btn-primary" onClick={playNextEpisode} style={{ padding: "8px 16px", fontSize: "0.88rem" }}>
            <Play size={14} fill="currentColor" /> Play Next
          </button>
        </div>
      )}

      {/* Floating Cinematic Overlay */}
      <div className={`player-overlay ${showControls ? "visible" : ""}`} onClick={(e) => e.stopPropagation()}>
        {/* Top Header Bar */}
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div>
            <h2 style={{ fontSize: "1.4rem", fontWeight: 700 }}>{title}</h2>
            <div style={{ fontSize: "0.85rem", color: "var(--text-secondary)", display: "flex", gap: 12, marginTop: 4 }}>
              {subtitle && <span>{subtitle}</span>}
              {subtitle && <span>•</span>}
              <span>{session.audio_tracks.find((t) => t.id === session.current_audio_track)?.name || "Default Audio"}</span>
              {subtitleOffsetMs !== 0 && (
                <span style={{ color: "var(--accent-primary)" }}>
                  Subtitles: {subtitleOffsetMs > 0 ? `+${subtitleOffsetMs}` : subtitleOffsetMs}ms
                </span>
              )}
            </div>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            {/* Snapshot Button */}
            <button className="btn-icon" onClick={captureSnapshot} title="Capture Frame Snapshot (S)">
              <Camera size={18} />
            </button>

            {/* Picture in Picture */}
            <button className="btn-icon" onClick={togglePiP} title="Picture in Picture (P)">
              <PictureInPicture size={18} />
            </button>

            {/* Close */}
            <button className="btn-icon" onClick={closePlayer} title="Close Player (Esc)">
              <X size={20} />
            </button>
          </div>
        </div>

        {/* Bottom Controls Area */}
        <div>
          {/* Timeline Scrubber */}
          <div className="player-timeline" onClick={handleTimelineClick} style={{ marginBottom: 16 }}>
            <div className="player-timeline-fill" style={{ width: `${progressPercent}%` }} />
          </div>

          {/* Controls Toolbar */}
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            {/* Left Controls: Play, Step, Skip, Timecode */}
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
              <button
                className="btn-icon"
                style={{ width: 44, height: 44, background: "var(--accent-primary)", border: "none" }}
                onClick={togglePlay}
                title={session.is_playing ? "Pause (Space)" : "Play (Space)"}
              >
                {session.is_playing ? <Pause size={22} fill="#fff" /> : <Play size={22} fill="#fff" />}
              </button>

              <button className="btn-icon" onClick={() => stepFrame(false)} title="Step 1 Frame Back (,)">
                <StepBack size={16} />
              </button>

              <button
                className="btn-icon"
                onClick={() => {
                  const prev = Math.max(0, session.position_seconds - 10);
                  if (videoRef.current) videoRef.current.currentTime = prev;
                  seek(prev);
                }}
                title="Rewind 10s (Left Arrow)"
              >
                <RotateCcw size={18} />
              </button>

              <button
                className="btn-icon"
                onClick={() => {
                  const next = session.position_seconds + 10;
                  if (videoRef.current) videoRef.current.currentTime = next;
                  seek(next);
                }}
                title="Forward 10s (Right Arrow)"
              >
                <RotateCw size={18} />
              </button>

              <button className="btn-icon" onClick={() => stepFrame(true)} title="Step 1 Frame Forward (.)">
                <StepForward size={16} />
              </button>

              {nextEpisode && (
                <button className="btn-icon" onClick={playNextEpisode} title="Next Episode">
                  <SkipForward size={18} color="var(--accent-primary)" />
                </button>
              )}

              {/* Time display */}
              <div style={{ fontSize: "0.9rem", fontWeight: 600, color: "var(--text-secondary)", marginLeft: 8 }}>
                <span style={{ color: "#fff" }}>{formatTime(session.position_seconds)}</span> / {formatTime(session.duration_seconds)}
              </div>
            </div>

            {/* Right Controls: Aspect Ratio, Audio, Subtitles, Speed, Volume, Fullscreen */}
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              {/* Aspect Ratio Menu */}
              <div style={{ position: "relative" }}>
                <button
                  className="btn-icon"
                  onClick={() => {
                    setShowAspectMenu(!showAspectMenu);
                    setShowAudioMenu(false);
                    setShowSubtitleMenu(false);
                    setShowSpeedMenu(false);
                    setShowSyncMenu(false);
                  }}
                  title="Aspect Ratio"
                >
                  <Crop size={18} />
                </button>
                {showAspectMenu && (
                  <div
                    className="glass-panel"
                    style={{
                      position: "absolute",
                      bottom: 50,
                      right: 0,
                      width: 160,
                      padding: 8,
                      display: "flex",
                      flexDirection: "column",
                      gap: 4,
                      zIndex: 100,
                    }}
                  >
                    <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", padding: "4px 8px", textTransform: "uppercase" }}>
                      Aspect Ratio
                    </div>
                    {(["fit", "16:9", "21:9", "4:3", "fill"] as AspectRatioMode[]).map((mode) => (
                      <button
                        key={mode}
                        className={`filter-pill ${aspectRatio === mode ? "active" : ""}`}
                        style={{ textAlign: "left", borderRadius: "var(--radius-sm)", fontSize: "0.82rem" }}
                        onClick={() => {
                          setAspectRatio(mode);
                          setShowAspectMenu(false);
                        }}
                      >
                        {mode === "fit" ? "Auto (Fit)" : mode === "fill" ? "Stretch (Fill)" : mode}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* Sync Tools (Audio & Subtitle Delay) */}
              <div style={{ position: "relative" }}>
                <button
                  className="btn-icon"
                  onClick={() => {
                    setShowSyncMenu(!showSyncMenu);
                    setShowAudioMenu(false);
                    setShowSubtitleMenu(false);
                    setShowSpeedMenu(false);
                    setShowAspectMenu(false);
                  }}
                  title="Track Synchronization Offset"
                >
                  <Sliders size={18} />
                </button>
                {showSyncMenu && (
                  <div
                    className="glass-panel"
                    style={{
                      position: "absolute",
                      bottom: 50,
                      right: 0,
                      width: 220,
                      padding: 12,
                      display: "flex",
                      flexDirection: "column",
                      gap: 8,
                      zIndex: 100,
                    }}
                  >
                    <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", textTransform: "uppercase" }}>
                      Subtitle Sync Offset
                    </div>
                    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                      <button
                        className="btn-icon"
                        style={{ width: 28, height: 28 }}
                        onClick={() => setSubtitleOffsetMs((prev) => prev - 250)}
                      >
                        -250ms
                      </button>
                      <span style={{ fontSize: "0.85rem", fontWeight: 700 }}>{subtitleOffsetMs}ms</span>
                      <button
                        className="btn-icon"
                        style={{ width: 28, height: 28 }}
                        onClick={() => setSubtitleOffsetMs((prev) => prev + 250)}
                      >
                        +250ms
                      </button>
                    </div>
                    <button
                      className="btn btn-secondary"
                      style={{ padding: "4px 8px", fontSize: "0.75rem" }}
                      onClick={() => setSubtitleOffsetMs(0)}
                    >
                      Reset Sync
                    </button>
                  </div>
                )}
              </div>

              {/* Audio Tracks Dropdown */}
              <div style={{ position: "relative" }}>
                <button
                  className="btn-icon"
                  onClick={() => {
                    setShowAudioMenu(!showAudioMenu);
                    setShowSubtitleMenu(false);
                    setShowSpeedMenu(false);
                    setShowAspectMenu(false);
                    setShowSyncMenu(false);
                  }}
                  title="Audio Tracks"
                >
                  <AudioLines size={18} />
                </button>
                {showAudioMenu && (
                  <div
                    className="glass-panel"
                    style={{
                      position: "absolute",
                      bottom: 50,
                      right: 0,
                      width: 220,
                      padding: 8,
                      display: "flex",
                      flexDirection: "column",
                      gap: 4,
                      zIndex: 100,
                    }}
                  >
                    <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", padding: "4px 8px", textTransform: "uppercase" }}>
                      Audio Stream
                    </div>
                    {session.audio_tracks.map((t) => (
                      <button
                        key={t.id}
                        className={`filter-pill ${session.current_audio_track === t.id ? "active" : ""}`}
                        style={{ textAlign: "left", borderRadius: "var(--radius-sm)", fontSize: "0.8rem" }}
                        onClick={() => {
                          selectAudio(t.id);
                          setShowAudioMenu(false);
                        }}
                      >
                        {t.name}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* Subtitles Dropdown */}
              <div style={{ position: "relative" }}>
                <button
                  className="btn-icon"
                  onClick={() => {
                    setShowSubtitleMenu(!showSubtitleMenu);
                    setShowAudioMenu(false);
                    setShowSpeedMenu(false);
                    setShowAspectMenu(false);
                    setShowSyncMenu(false);
                  }}
                  title="Subtitles"
                >
                  <Subtitles size={18} />
                </button>
                {showSubtitleMenu && (
                  <div
                    className="glass-panel"
                    style={{
                      position: "absolute",
                      bottom: 50,
                      right: 0,
                      width: 240,
                      padding: 8,
                      display: "flex",
                      flexDirection: "column",
                      gap: 4,
                      zIndex: 100,
                    }}
                  >
                    <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", padding: "4px 8px", textTransform: "uppercase" }}>
                      Subtitles
                    </div>
                    <button
                      className={`filter-pill ${session.current_subtitle_track === null ? "active" : ""}`}
                      style={{ textAlign: "left", borderRadius: "var(--radius-sm)", fontSize: "0.8rem" }}
                      onClick={() => {
                        selectSubtitle(null);
                        setShowSubtitleMenu(false);
                      }}
                    >
                      Off
                    </button>
                    {session.subtitle_tracks.map((t) => (
                      <button
                        key={t.id}
                        className={`filter-pill ${session.current_subtitle_track === t.id ? "active" : ""}`}
                        style={{ textAlign: "left", borderRadius: "var(--radius-sm)", fontSize: "0.8rem" }}
                        onClick={() => {
                          selectSubtitle(t.id);
                          setShowSubtitleMenu(false);
                        }}
                      >
                        {t.name}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* Playback Speed */}
              <div style={{ position: "relative" }}>
                <button
                  className="btn-icon"
                  onClick={() => {
                    setShowSpeedMenu(!showSpeedMenu);
                    setShowAudioMenu(false);
                    setShowSubtitleMenu(false);
                    setShowAspectMenu(false);
                    setShowSyncMenu(false);
                  }}
                  title="Speed"
                >
                  <Gauge size={18} />
                </button>
                {showSpeedMenu && (
                  <div
                    className="glass-panel"
                    style={{
                      position: "absolute",
                      bottom: 50,
                      right: 0,
                      width: 140,
                      padding: 8,
                      display: "flex",
                      flexDirection: "column",
                      gap: 4,
                      zIndex: 100,
                    }}
                  >
                    {[0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0].map((s) => (
                      <button
                        key={s}
                        className={`filter-pill ${session.playback_speed === s ? "active" : ""}`}
                        style={{ textAlign: "center", borderRadius: "var(--radius-sm)", fontSize: "0.8rem" }}
                        onClick={() => {
                          setSpeed(s);
                          if (videoRef.current) videoRef.current.playbackRate = s;
                          setShowSpeedMenu(false);
                        }}
                      >
                        {s}x
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* Volume Slider with 200% Boost */}
              <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                <button className="btn-icon" onClick={toggleMute} title={session.is_muted ? "Unmute (M)" : "Mute (M)"}>
                  {session.is_muted || customVolume === 0 ? (
                    <VolumeX size={18} />
                  ) : customVolume > 100 ? (
                    <Volume2 size={18} color="var(--accent-gold)" />
                  ) : customVolume > 50 ? (
                    <Volume2 size={18} />
                  ) : (
                    <Volume1 size={18} />
                  )}
                </button>
                <input
                  type="range"
                  min={0}
                  max={200}
                  value={session.is_muted ? 0 : customVolume}
                  onChange={(e) => {
                    const v = Number(e.target.value);
                    setCustomVolume(v);
                    setVolume(Math.min(100, v));
                  }}
                  style={{ width: 80, accentColor: customVolume > 100 ? "var(--accent-gold)" : "var(--accent-primary)" }}
                  title={`Volume: ${customVolume}%`}
                />
              </div>

              {/* Fullscreen Button */}
              <button className="btn-icon" onClick={toggleFullscreen} title="Fullscreen (F)">
                {isFullscreen ? <Minimize size={18} /> : <Maximize size={18} />}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
