import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "../store/useStore";

export const usePlayback = () => {
    const {
        playlist,
        currentIndex,
        qualityMode,
        aspectMode,
        setPlaylist,
        setCurrentIndex,
        setIsPlaying,
        setQualityMode,
        setAspectMode,
        volume,
        setVolume,
        isMuted,
        setIsMuted
    } = useStore();

    const currentFile = currentIndex !== null ? playlist[currentIndex] : null;

    const handleSetVolume = async (val: number) => {
        setVolume(val);
        await invoke("set_volume", { volume: isMuted ? 0 : val });
    };

    const handleToggleMute = async () => {
        const nextMuted = !isMuted;
        setIsMuted(nextMuted);
        await invoke("set_volume", { volume: nextMuted ? 0 : volume });
    };

    const handleOpenFile = async (path: string) => {
        try {
            await invoke("open_video", { path });
            setIsPlaying(true);
        } catch (err) {
            console.error("Failed to open video:", err);
        }
    };

    const handleImport = async () => {
        const selected = await open({
            multiple: true,
            filters: [
                {
                    name: "Media",
                    extensions: ["mp4", "mkv", "avi", "mov", "webm", "mp3", "wav", "flac", "m4a", "jpg", "jpeg", "png", "webp", "bmp", "tiff", "tif"],
                },
            ],
        });

        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];
            const newItems = paths.map(p => ({
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
    };

    const selectMedia = (index: number) => {
        setCurrentIndex(index);
        handleOpenFile(playlist[index].path);
    };

    const handleToggleQuality = async () => {
        const modes: ("Native" | "Fast" | "Proxy")[] = ["Native", "Fast", "Proxy"];
        const nextIndex = (modes.indexOf(qualityMode) + 1) % modes.length;
        const nextMode = modes[nextIndex];
        setQualityMode(nextMode);
        await invoke("set_quality", { mode: nextMode });

        if (currentFile) {
            handleOpenFile(currentFile.path);
        }
    };

    const handleTogglePlayback = async () => {
        if (currentIndex === null) return;
        try {
            const playing = await invoke<boolean>("toggle_playback");
            setIsPlaying(playing);
        } catch (err) {
            console.error("Failed to toggle playback:", err);
        }
    };

    const handleSeek = async (time: number) => {
        try {
            await invoke("seek_video", { time });
            // Optimistically update store if needed, though event listener handles it
        } catch (err) {
            console.error("Failed to seek:", err);
        }
    };

    const handleToggleAspect = async () => {
        const modes: ("Fit" | "Stretch" | "Cinema" | "Classic" | "Wide")[] = ["Fit", "Stretch", "Cinema", "Classic", "Wide"];
        const nextIndex = (modes.indexOf(aspectMode) + 1) % modes.length;
        const nextMode = modes[nextIndex];
        setAspectMode(nextMode);
        await invoke("set_aspect_ratio", { mode: nextMode });
    };

    return {
        handleImport,
        handleOpenFile,
        handleToggleQuality,
        handleTogglePlayback,
        handleToggleAspect,
        handleSetVolume,
        handleToggleMute,
        selectMedia,
        handleSeek,
        currentFile,
        volume,
        isMuted
    };
};
