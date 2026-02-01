import { create } from "zustand";

interface PlaybackState {
    isPlaying: boolean;
    currentTime: number;
    duration: number;
    qualityMode: "Native" | "Fast" | "Proxy";
    aspectMode: "Fit" | "Stretch" | "Cinema" | "Classic" | "Wide";
    playlist: string[];
    currentIndex: number | null;

    // Actions
    setIsPlaying: (playing: boolean) => void;
    setCurrentTime: (time: number) => void;
    setDuration: (duration: number) => void;
    setQualityMode: (mode: "Native" | "Fast" | "Proxy") => void;
    setAspectMode: (mode: "Fit" | "Stretch" | "Cinema" | "Classic" | "Wide") => void;
    setPlaylist: (playlist: string[] | ((prev: string[]) => string[])) => void;
    setCurrentIndex: (index: number | null) => void;
}

export const useStore = create<PlaybackState>((set) => ({
    isPlaying: false,
    currentTime: 0,
    duration: 0,
    qualityMode: "Native",
    aspectMode: "Fit",
    playlist: [],
    currentIndex: null,

    setIsPlaying: (isPlaying) => set({ isPlaying }),
    setCurrentTime: (currentTime) => set({ currentTime }),
    setDuration: (duration) => set({ duration }),
    setQualityMode: (qualityMode) => set({ qualityMode }),
    setAspectMode: (aspectMode) => set({ aspectMode }),
    setPlaylist: (playlist) => set((state) => ({
        playlist: typeof playlist === "function" ? playlist(state.playlist) : playlist
    })),
    setCurrentIndex: (currentIndex) => set({ currentIndex }),
}));
