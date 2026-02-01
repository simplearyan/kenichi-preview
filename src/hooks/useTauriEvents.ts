import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "../store/useStore";
import { usePlayback } from "./usePlayback";

export const useTauriEvents = () => {
    const { setPlaylist, setCurrentIndex, setCurrentTime, setDuration, currentIndex, isPlaying, setPlaybackStatus, setIsPlaying } = useStore();
    const { handleOpenFile, handleTogglePlayback } = usePlayback();

    useEffect(() => {
        const unlistenDragDrop = listen("tauri://drag-drop", async (event: any) => {
            const paths = event.payload.paths;
            if (paths && paths.length > 0) {
                const newItems = paths.map((p: string) => ({
                    path: p,
                    name: p.split(/[\\/]/).pop() || p
                }));

                setPlaylist(prev => {
                    const updatedPlaylist = [...prev, ...newItems];
                    if (currentIndex === null) {
                        setCurrentIndex(prev.length);
                        handleOpenFile(newItems[0].path);
                    }
                    return updatedPlaylist;
                });
            }
        });

        const unlistenPlayback = listen("playback-update", (event: any) => {
            setCurrentTime(event.payload.current_time);
            setDuration(event.payload.duration);
            if (event.payload.status) {
                setPlaybackStatus(event.payload.status);
                setIsPlaying(event.payload.status === "Playing");
            }
        });

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.code === "Space") {
                e.preventDefault();
                handleTogglePlayback();
            }
        };

        window.addEventListener("keydown", handleKeyDown);

        return () => {
            unlistenDragDrop.then(f => f());
            unlistenPlayback.then(f => f());
            window.removeEventListener("keydown", handleKeyDown);
        };
    }, [currentIndex, isPlaying]);
};
