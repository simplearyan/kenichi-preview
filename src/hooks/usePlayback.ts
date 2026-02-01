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
        setAspectMode
    } = useStore();

    const currentFile = currentIndex !== null ? playlist[currentIndex] : null;

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
                    extensions: ["mp4", "mkv", "avi", "mov", "webm", "jpg", "png", "webp"],
                },
            ],
        });

        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];
            setPlaylist(prev => {
                const newPlaylist = [...prev, ...paths];
                if (currentIndex === null) {
                    setCurrentIndex(prev.length);
                    handleOpenFile(paths[0]);
                }
                return newPlaylist;
            });
        }
    };

    const selectMedia = (index: number) => {
        setCurrentIndex(index);
        handleOpenFile(playlist[index]);
    };

    const handleToggleQuality = async () => {
        const modes: ("Native" | "Fast" | "Proxy")[] = ["Native", "Fast", "Proxy"];
        const nextIndex = (modes.indexOf(qualityMode) + 1) % modes.length;
        const nextMode = modes[nextIndex];
        setQualityMode(nextMode);
        await invoke("set_quality", { mode: nextMode });

        if (currentFile) {
            handleOpenFile(currentFile);
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
        selectMedia,
        currentFile
    };
};
