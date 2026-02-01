import { Maximize2, Subtitles, Play, Pause, FastForward, ShieldCheck, Zap } from "lucide-react";
import { useStore } from "../store/useStore";
import { usePlayback } from "../hooks/usePlayback";
import { formatTime } from "../utils/format";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { VolumeControl } from "./VolumeControl";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const ControlBar = () => {
    // Use selectors for high-frequency updates to prevent unnecessary re-renders
    const currentTime = useStore((state) => state.currentTime);
    const duration = useStore((state) => state.duration);
    const isPlaying = useStore((state) => state.isPlaying);
    const qualityMode = useStore((state) => state.qualityMode);
    const aspectMode = useStore((state) => state.aspectMode);
    const playlist = useStore((state) => state.playlist);
    const currentIndex = useStore((state) => state.currentIndex);

    const { handleTogglePlayback, handleToggleQuality, handleToggleAspect } = usePlayback();

    const currentFile = currentIndex !== null ? playlist[currentIndex] : null;

    return (
        <footer className="h-20 glass-panel border-t border-white/5 px-6 flex items-center justify-between z-50">
            <div className="flex items-center gap-6 w-1/3">
                <div className="flex flex-col">
                    <span className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-1">Active Preview</span>
                    <div className="text-sm font-bold text-white truncate max-w-[200px]">
                        {currentFile ? currentFile.split(/[\\/]/).pop() : "Idle"}
                    </div>
                </div>
                <div className="flex flex-col ml-10">
                    <span className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-1">Time / Duration</span>
                    <div className="text-sm font-mono font-bold text-brand-yellow tabular-nums">
                        {formatTime(currentTime)} <span className="text-zinc-600 font-normal">/</span> {formatTime(duration)}
                    </div>
                </div>
            </div>

            <div className="flex items-center gap-4 no-drag">
                <button
                    onClick={handleToggleAspect}
                    className={cn(
                        "p-2.5 rounded-xl transition-all relative group",
                        aspectMode === "Fit" ? "text-zinc-500 hover:bg-white/5" : "text-brand-yellow bg-brand-yellow/10 border border-brand-yellow/20"
                    )}
                    title={`Aspect Ratio: ${aspectMode}`}
                >
                    <Maximize2 className="w-5 h-5" />
                    {aspectMode !== "Fit" && (
                        <span className="absolute -top-1 -right-1 flex h-2 w-2">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-brand-yellow opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2 w-2 bg-brand-yellow"></span>
                        </span>
                    )}
                </button>

                <button className="p-2.5 rounded-xl hover:bg-white/5 text-zinc-500 transition-all">
                    <Subtitles className="w-5 h-5" />
                </button>

                <button
                    onClick={handleTogglePlayback}
                    className="w-12 h-12 flex items-center justify-center rounded-2xl bg-brand-yellow text-pro-black hover:scale-105 active:scale-95 shadow-lg shadow-brand-yellow/20 transition-all"
                >
                    {isPlaying ? (
                        <Pause className="w-5 h-5 fill-current" />
                    ) : (
                        <Play className="w-5 h-5 fill-current ml-0.5" />
                    )}
                </button>

                {/* Volume Control */}
                <VolumeControl />

                <button className="p-2.5 rounded-xl hover:bg-white/5 text-zinc-500 transition-all">
                    <FastForward className="w-5 h-5" />
                </button>
            </div>

            <div className="flex items-center justify-end gap-3 w-1/3">
                <button
                    onClick={handleToggleQuality}
                    className={cn(
                        "flex items-center gap-2 px-4 py-2 rounded-xl border transition-all duration-300 no-drag",
                        qualityMode === "Native"
                            ? "bg-white/5 border-white/10 hover:border-white/20 text-white"
                            : "bg-brand-yellow text-pro-black border-brand-yellow shadow-lg shadow-brand-yellow/20"
                    )}
                    title={`Quality: ${qualityMode}`}
                >
                    {qualityMode === "Native" && <ShieldCheck className="w-4 h-4" />}
                    {qualityMode === "Fast" && <Zap className="w-4 h-4" />}
                    {qualityMode === "Proxy" && <FastForward className="w-4 h-4" />}
                    <span className="text-xs font-bold tracking-tight uppercase">
                        {qualityMode === "Native" ? "High Fidelity" : qualityMode}
                    </span>
                </button>
            </div>
        </footer>
    );
};
