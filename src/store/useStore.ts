import { create } from "zustand";

export interface MediaItem {
    path: string;
    name: string;
    type?: 'Video' | 'Audio' | 'Image';
    thumbnail?: string;
    duration?: number;
    processing?: boolean;
    processed?: boolean;  // To prevent infinite re-processing
    size?: number;        // bytes
    width?: number;
    height?: number;
    sampleRate?: number;  // Hz
    bitrate?: number;     // bps
    container?: string;   // e.g. 'mp4', 'mov'
    fps?: number;
    videoCodec?: string;
    audioCodec?: string;
    channels?: number;
    pixelFormat?: string;
    audioLayout?: string;  // e.g. "stereo", "mono", "5.1"
    videoProfile?: string; // e.g. "Main 4.1"
    audioDepth?: string;   // e.g. "16-bit", "fltp"
}

export type PlaybackStatus = "Playing" | "Paused" | "Buffering" | "Finished" | "Error";

interface PlaybackState {
    isPlaying: boolean;
    currentTime: number;
    duration: number;
    qualityMode: "Native" | "Fast" | "Proxy";
    aspectMode: "Fit" | "Stretch" | "Cinema" | "Classic" | "Wide";
    playlist: MediaItem[];
    currentIndex: number | null;
    isRendererReady: boolean;
    volume: number;
    isMuted: boolean;
    playbackStatus: PlaybackStatus;

    // Actions
    setIsPlaying: (playing: boolean) => void;
    setCurrentTime: (time: number) => void;
    setDuration: (duration: number) => void;
    setQualityMode: (mode: "Native" | "Fast" | "Proxy") => void;
    setAspectMode: (mode: "Fit" | "Stretch" | "Cinema" | "Classic" | "Wide") => void;
    setPlaylist: (playlist: MediaItem[] | ((prev: MediaItem[]) => MediaItem[])) => void;
    updateMediaItem: (index: number, updates: Partial<MediaItem>) => void;
    setCurrentIndex: (index: number | null) => void;
    setIsRendererReady: (ready: boolean) => void;
    setVolume: (volume: number) => void;
    setIsMuted: (muted: boolean) => void;
    setPlaybackStatus: (status: PlaybackStatus) => void;
}

export const useStore = create<PlaybackState>((set) => ({
    isPlaying: false,
    currentTime: 0,
    duration: 0,
    qualityMode: "Native",
    aspectMode: "Fit",
    playlist: [],
    currentIndex: null,
    isRendererReady: false,
    volume: 1.0,
    isMuted: false,
    playbackStatus: "Paused",

    setIsPlaying: (isPlaying) => set({ isPlaying }),
    setPlaybackStatus: (playbackStatus) => set({ playbackStatus }),
    setCurrentTime: (currentTime) => set({ currentTime }),
    setDuration: (duration) => set({ duration }),
    setQualityMode: (qualityMode) => set({ qualityMode }),
    setAspectMode: (aspectMode) => set({ aspectMode }),
    setPlaylist: (playlist) => set((state) => ({
        playlist: typeof playlist === "function" ? playlist(state.playlist) : playlist
    })),
    updateMediaItem: (index, updates) => set((state) => {
        const newPlaylist = [...state.playlist];
        if (newPlaylist[index]) {
            newPlaylist[index] = { ...newPlaylist[index], ...updates };
        }
        return { playlist: newPlaylist };
    }),
    setCurrentIndex: (currentIndex) => set({ currentIndex }),
    setIsRendererReady: (isRendererReady) => set({ isRendererReady }),
    setVolume: (volume) => set({ volume }),
    setIsMuted: (isMuted) => set({ isMuted }),
}));
