import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "../store/useStore";
import { usePlayback } from "./usePlayback";

export const useTauriEvents = () => {
    const { setPlaylist, setCurrentIndex, setCurrentTime, setDuration, currentIndex, setPlaybackStatus, setIsPlaying } = useStore();
    const { handleOpenFile, handleTogglePlayback } = usePlayback();

    // Stable listener for Backend Events (Run once)
    useEffect(() => {
        const unlistenDragDropPromise = listen("tauri://drag-drop", async (event: any) => {
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

        const unlistenPlaybackPromise = listen("playback-update", (event: any) => {
            setCurrentTime(event.payload.current_time);
            setDuration(event.payload.duration);
            if (event.payload.status) {
                setPlaybackStatus(event.payload.status);
                setIsPlaying(event.payload.status === "Playing");
            }
        });

        return () => {
            unlistenDragDropPromise.then(f => f());
            unlistenPlaybackPromise.then(f => f());
        };
    }, []); // Empty dependency array: Setup once!

    // Keydown Handler (Dependent on current state/callbacks)
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.code === "Space") {
                e.preventDefault();
                handleTogglePlayback();
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [handleTogglePlayback]);
};
