import { Minus, Square, Play, X, Zap, Timer } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { usePlayback } from "../../hooks/usePlayback";

const appWindow = getCurrentWindow();

export const TitleBar = () => {
    const { syncMode, handleSetSyncMode } = usePlayback();

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
