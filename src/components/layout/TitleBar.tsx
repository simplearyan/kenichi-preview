import { Minus, Square, Play, X, Zap, Timer, Info, Camera, Download } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { usePlayback } from "../../hooks/usePlayback";
import { useStore } from "../../store/useStore";
import { useExportFrame } from "../../hooks/useExportFrame";
import { useTrimActions } from "../../hooks/useTrimActions";

const appWindow = getCurrentWindow();

export const TitleBar = () => {
    const { syncMode, handleSetSyncMode } = usePlayback();
    const showMetadata = useStore((state) => state.showMetadata);
    const setShowMetadata = useStore((state) => state.setShowMetadata);
    const { handleExportFrame, isExporting } = useExportFrame();

    // Check if current item is video to enable button
    const playlist = useStore((state) => state.playlist);
    const currentIndex = useStore((state) => state.currentIndex);
    const isVideo = currentIndex !== null && playlist[currentIndex]?.type === 'Video';

    // Trim Export
    const { handleExportClip, isExportingClip, trimStart, trimEnd } = useTrimActions();
    const hasTrim = trimStart !== undefined || trimEnd !== undefined;

    return (
        <header className="h-10 flex items-center justify-between glass-panel drag-region px-4 z-50">
            <div className="flex items-center gap-2">
                <div className="w-5 h-5 bg-brand-yellow rounded-sm flex items-center justify-center">
                    <Play className="w-3 h-3 text-pro-black fill-current" />
                </div>
                <span className="text-sm font-bold tracking-tight text-white uppercase italic">
                    Kenichi<span className="text-brand-yellow">Preview</span>
                </span>
            </div>

            <div className="flex items-center no-drag">
                {/* Export Clip (Only visible if Trim Active) */}
                <button
                    onClick={handleExportClip}
                    disabled={!isVideo || isExportingClip || !hasTrim}
                    className={`flex items-center gap-1.5 px-2 py-1 hover:bg-white/5 transition-colors mr-1 rounded-md ${isExportingClip ? 'animate-pulse text-pink-500' : 'text-zinc-400'} disabled:opacity-30 disabled:hidden`}
                    title="Export Trimmed Clip"
                >
                    <Download className="w-4 h-4" />
                    <span className="text-xs font-bold uppercase text-pink-500">Clip</span>
                </button>

                <div className="w-px h-4 bg-white/10 mx-2" />

                {/* Export Frame */}
                <button
                    onClick={handleExportFrame}
                    disabled={!isVideo || isExporting}
                    className={`p-2 hover:bg-white/5 transition-colors mr-1 rounded-md ${isExporting ? 'animate-pulse text-brand-yellow' : 'text-zinc-400'} disabled:opacity-30 disabled:cursor-not-allowed`}
                    title="Export Current Frame (Snapshot)"
                >
                    <Camera className="w-4 h-4" />
                </button>

                {/* Metadata Toggle */}
                {/* Metadata Toggle */}
                <button
                    onClick={() => setShowMetadata(!showMetadata)}
                    className={`p-2 hover:bg-white/5 transition-colors mr-2 rounded-md ${showMetadata ? 'text-brand-yellow bg-brand-yellow/10' : 'text-zinc-400'}`}
                    title="Toggle Advanced Metadata"
                >
                    <Info className="w-4 h-4" />
                </button>

                {/* Sync Mode Toggle */}
                <div className="flex items-center gap-1 mr-4 border-r border-white/10 pr-4 h-5">
                    <button
                        onClick={() => handleSetSyncMode(syncMode === "Realtime" ? "Fixed" : "Realtime")}
                        className="flex items-center gap-1.5 px-2 py-1 rounded hover:bg-white/5 transition-colors text-[10px] font-mono tracking-wider uppercase"
                        title="Toggle Playback Sync Mode"
                    >
                        {syncMode === "Realtime" ? (
                            <>
                                <Zap className="w-3 h-3 text-brand-yellow fill-brand-yellow/20" />
                                <span className="text-brand-yellow/90 font-semibold">Realtime</span>
                            </>
                        ) : (
                            <>
                                <Timer className="w-3 h-3 text-zinc-500" />
                                <span className="text-zinc-500 font-medium">Fixed 30ms</span>
                            </>
                        )}
                    </button>
                </div>

                <button onClick={() => appWindow.minimize()} className="p-2 hover:bg-white/5 transition-colors">
                    <Minus className="w-4 h-4" />
                </button>
                <button onClick={() => appWindow.maximize()} className="p-2 hover:bg-white/5 transition-colors">
                    <Square className="w-3.5 h-3.5" />
                </button>
                <button onClick={() => appWindow.close()} className="p-2 hover:bg-red-500/80 hover:text-white transition-colors">
                    <X className="w-4 h-4" />
                </button>
            </div>
        </header >
    );
};
