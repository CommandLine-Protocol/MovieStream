import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from "react";
import { Movie, PlaybackSession, TvEpisode } from "../types";
import * as ipc from "../ipc";
import { useToast } from "./ToastContext";

interface PlaybackContextType {
  session: PlaybackSession | null;
  activeMovie: Movie | null;
  isPlayerOpen: boolean;
  showResumePrompt: boolean;
  resumePosition: number;
  resumeCountdown: number;
  nextEpisode: TvEpisode | null;
  startMovie: (movie: Movie, mediaId?: string) => Promise<void>;
  startEpisode: (episodeId: string, mediaId: string) => Promise<void>;
  acceptResume: () => Promise<void>;
  restartFromBeginning: () => Promise<void>;
  playNextEpisode: () => Promise<void>;
  play: () => Promise<void>;
  pause: () => Promise<void>;
  togglePlay: () => Promise<void>;
  stop: () => Promise<void>;
  closePlayer: () => Promise<void>;
  seek: (seconds: number) => Promise<void>;
  setVolume: (level: number) => Promise<void>;
  toggleMute: () => Promise<void>;
  setSpeed: (speed: number) => Promise<void>;
  selectAudio: (trackId: string) => Promise<void>;
  selectSubtitle: (trackId: string | null) => Promise<void>;
  loadSubtitleFile: (path: string) => Promise<void>;
}

const PlaybackContext = createContext<PlaybackContextType | undefined>(undefined);

export const PlaybackProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [session, setSession] = useState<PlaybackSession | null>(null);
  const [activeMovie, setActiveMovie] = useState<Movie | null>(null);
  const [isPlayerOpen, setIsPlayerOpen] = useState<boolean>(false);
  const [showResumePrompt, setShowResumePrompt] = useState<boolean>(false);
  const [resumePosition, setResumePosition] = useState<number>(0);
  const [resumeCountdown, setResumeCountdown] = useState<number>(15);
  const [nextEpisode, setNextEpisode] = useState<TvEpisode | null>(null);

  const { showToast } = useToast();
  const timerRef = useRef<number | null>(null);
  const countdownIntervalRef = useRef<number | null>(null);

  // Position tick simulator / synchronizer when playing
  useEffect(() => {
    if (isPlayerOpen && session?.is_playing) {
      timerRef.current = window.setInterval(() => {
        setSession((prev) => {
          if (!prev || !prev.is_playing) return prev;
          const nextPos = prev.position_seconds + 1;
          if (nextPos % 5 === 0) {
            ipc.recordPosition(nextPos).catch(console.error);
          }
          return { ...prev, position_seconds: nextPos };
        });
      }, 1000);
    } else {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    }

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [isPlayerOpen, session?.is_playing]);

  // Check for next episode when playback reaches near end
  useEffect(() => {
    if (session?.media_type === "episode" && session.duration_seconds > 0) {
      if (session.position_seconds >= session.duration_seconds * 0.95 || session.duration_seconds - session.position_seconds <= 90) {
        ipc.getNextEpisode().then((ep) => {
          if (ep) setNextEpisode(ep);
        }).catch(() => {});
      }
    }
  }, [session?.position_seconds, session?.duration_seconds, session?.media_type]);

  // 15-second visual countdown timer for resume prompt
  useEffect(() => {
    if (showResumePrompt) {
      setResumeCountdown(15);
      if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);

      countdownIntervalRef.current = window.setInterval(() => {
        setResumeCountdown((prev) => {
          if (prev <= 1) {
            if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
            // Auto resume when timer hits 0
            acceptResume();
            return 0;
          }
          return prev - 1;
        });
      }, 1000);
    } else {
      if (countdownIntervalRef.current) {
        clearInterval(countdownIntervalRef.current);
        countdownIntervalRef.current = null;
      }
    }

    return () => {
      if (countdownIntervalRef.current) {
        clearInterval(countdownIntervalRef.current);
        countdownIntervalRef.current = null;
      }
    };
  }, [showResumePrompt]);

  // Listen for backend position and error events
  useEffect(() => {
    let unlistenPos: (() => void) | undefined;
    let unlistenErr: (() => void) | undefined;

    ipc.onPlaybackPosition((payload) => {
      setSession((prev) => (prev ? { ...prev, position_seconds: payload.position_seconds } : prev));
    }).then((u) => {
      unlistenPos = u;
    });

    ipc.onPlaybackError((payload) => {
      showToast({
        title: "Playback Error",
        message: payload.message,
        type: "error",
      });
    }).then((u) => {
      unlistenErr = u;
    });

    return () => {
      if (unlistenPos) unlistenPos();
      if (unlistenErr) unlistenErr();
    };
  }, [showToast]);

  const startMovie = useCallback(
    async (movie: Movie, mediaId?: string) => {
      try {
        const movieWithMedia = await ipc.getMovie(movie.id);
        if (!movieWithMedia || movieWithMedia.media.length === 0) {
          showToast({
            title: "Media Unavailable",
            message: "No playable media files found for this movie.",
            type: "error",
          });
          return;
        }

        const selectedMedia = mediaId
          ? movieWithMedia.media.find((m) => m.id === mediaId) || movieWithMedia.media[0]
          : movieWithMedia.media[0];

        if (selectedMedia.availability === "unavailable") {
          showToast({
            title: "File Disconnected",
            message: "The drive containing this movie is currently disconnected.",
            type: "warning",
          });
          return;
        }

        const sess = await ipc.startPlayback(movie.id, selectedMedia.id);
        setActiveMovie(movie);
        setSession(sess);
        setIsPlayerOpen(true);
        setNextEpisode(null);

        if (sess.requires_resume_prompt) {
          setShowResumePrompt(true);
          setResumePosition(sess.resume_position_seconds);
        } else {
          setShowResumePrompt(false);
        }
      } catch (err) {
        showToast({
          title: "Playback Failed",
          message: String(err),
          type: "error",
        });
      }
    },
    [showToast]
  );

  const startEpisode = useCallback(
    async (episodeId: string, mediaId: string) => {
      try {
        const sess = await ipc.startEpisodePlayback(episodeId, mediaId);
        setActiveMovie(null);
        setSession(sess);
        setIsPlayerOpen(true);
        setNextEpisode(null);

        if (sess.requires_resume_prompt) {
          setShowResumePrompt(true);
          setResumePosition(sess.resume_position_seconds);
        } else {
          setShowResumePrompt(false);
        }
      } catch (err) {
        showToast({
          title: "Playback Failed",
          message: String(err),
          type: "error",
        });
      }
    },
    [showToast]
  );

  const acceptResume = async () => {
    try {
      if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
      await ipc.resumeAt(resumePosition);
      setShowResumePrompt(false);
      setSession((prev) =>
        prev ? { ...prev, position_seconds: resumePosition, is_playing: true, requires_resume_prompt: false } : prev
      );
    } catch (err) {
      console.error(err);
    }
  };

  const restartFromBeginning = async () => {
    try {
      if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
      await ipc.seek(0);
      await ipc.play();
      setShowResumePrompt(false);
      setSession((prev) => (prev ? { ...prev, position_seconds: 0, is_playing: true, requires_resume_prompt: false } : prev));
    } catch (err) {
      console.error(err);
    }
  };

  const playNextEpisode = async () => {
    if (!nextEpisode) return;
    try {
      // Find media for next episode
      if (session?.series_id) {
        const details = await ipc.getSeriesDetails(session.series_id);
        const nextEpWithMedia = details?.seasons
          .flatMap((s) => s.episodes)
          .find((e) => e.episode.id === nextEpisode.id);

        if (nextEpWithMedia?.media_id) {
          await startEpisode(nextEpisode.id, nextEpWithMedia.media_id);
        } else {
          showToast({ title: "Next Episode", message: "No local media file found for next episode", type: "info" });
        }
      }
    } catch (err) {
      console.error("Failed to play next episode:", err);
    }
  };

  const play = async () => {
    await ipc.play();
    setSession((prev) => (prev ? { ...prev, is_playing: true } : prev));
  };

  const pause = async () => {
    await ipc.pause();
    setSession((prev) => (prev ? { ...prev, is_playing: false } : prev));
  };

  const togglePlay = async () => {
    if (session?.is_playing) {
      await pause();
    } else {
      await play();
    }
  };

  const stop = async () => {
    await ipc.stop();
    setSession((prev) => (prev ? { ...prev, is_playing: false, position_seconds: 0 } : prev));
  };

  const closePlayer = async () => {
    await stop();
    setIsPlayerOpen(false);
    setShowResumePrompt(false);
    setActiveMovie(null);
    setNextEpisode(null);
  };

  const seek = async (seconds: number) => {
    await ipc.seek(seconds);
    setSession((prev) => (prev ? { ...prev, position_seconds: seconds } : prev));
  };

  const setVolume = async (level: number) => {
    await ipc.setVolume(level);
    setSession((prev) => (prev ? { ...prev, volume: level, is_muted: false } : prev));
  };

  const toggleMute = async () => {
    if (!session) return;
    const nextMute = !session.is_muted;
    await ipc.setMute(nextMute);
    setSession((prev) => (prev ? { ...prev, is_muted: nextMute } : prev));
  };

  const setSpeed = async (speed: number) => {
    await ipc.setPlaybackSpeed(speed);
    setSession((prev) => (prev ? { ...prev, playback_speed: speed } : prev));
  };

  const selectAudio = async (trackId: string) => {
    await ipc.selectAudioTrack(trackId);
    setSession((prev) => (prev ? { ...prev, current_audio_track: trackId } : prev));
  };

  const selectSubtitle = async (trackId: string | null) => {
    await ipc.selectSubtitleTrack(trackId);
    setSession((prev) => (prev ? { ...prev, current_subtitle_track: trackId } : prev));
  };

  const loadSubtitleFile = async (path: string) => {
    await ipc.loadExternalSubtitle(path);
    const updated = await ipc.getActiveSession();
    if (updated) {
      setSession(updated);
    }
  };

  return (
    <PlaybackContext.Provider
      value={{
        session,
        activeMovie,
        isPlayerOpen,
        showResumePrompt,
        resumePosition,
        resumeCountdown,
        nextEpisode,
        startMovie,
        startEpisode,
        acceptResume,
        restartFromBeginning,
        playNextEpisode,
        play,
        pause,
        togglePlay,
        stop,
        closePlayer,
        seek,
        setVolume,
        toggleMute,
        setSpeed,
        selectAudio,
        selectSubtitle,
        loadSubtitleFile,
      }}
    >
      {children}
    </PlaybackContext.Provider>
  );
};

export const usePlayback = (): PlaybackContextType => {
  const context = useContext(PlaybackContext);
  if (!context) {
    throw new Error("usePlayback must be used within a PlaybackProvider");
  }
  return context;
};
