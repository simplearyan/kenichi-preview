import { Maximize2, Subtitles, Play, Pause, FastForward, ShieldCheck, Zap } from "lucide-react";
import { useState, useEffect } from "react";
import { useStore } from "../../store/useStore";
import { usePlayback } from "../../hooks/usePlayback";
import { formatTime } from "../../utils/format";
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

    const { handleTogglePlayback, handleToggleQuality, handleToggleAspect, handleSeek } = usePlayback();

    // Local state for smooth seeking
    const [isDragging, setIsDragging] = useState(false);
    const [sliderValue, setSliderValue] = useState(0);
    const [isDebouncing, setIsDebouncing] = useState(false);

    // Effect: Exit debounce early if backend catches up
    useEffect(() => {
        if (isDebouncing) {
            const diff = Math.abs(currentTime - sliderValue);
            // Only snap back if we remain extremely close (50ms) to avoid visual jitter
            // OR if time has passed the target (we caught up)
            if (diff < 0.1 || currentTime > sliderValue) {
                setIsDebouncing(false);
            }
        }
    }, [currentTime, isDebouncing, sliderValue]);

    const currentFile = currentIndex !== null ? playlist[currentIndex] : null;

    return (
        <footer className="glass-panel border-t border-white/5 z-50 flex flex-col relative bg-zinc-950/80 backdrop-blur-xl">
            {/* Progress Bar - Full Width Top */}
            <div className="w-full h-1.5 relative group cursor-pointer">
                {/* Background Track */}
                <div className="absolute top-0 left-0 right-0 bottom-0 bg-white/5 group-hover:bg-white/10 transition-colors" />

                {/* Progress Fill */}
                <div
                    className="absolute top-0 left-0 bottom-0 bg-brand-yellow/80 group-hover:bg-brand-yellow transition-all duration-75 ease-linear"
                    style={{ width: `${((isDragging ? sliderValue : currentTime) / (duration || 1)) * 100}%` }}
                />

                {/* Range Input for Seek */}
                <input
                    type="range"
                    min="0"
                    max={duration || 100}
                    step="0.01"
                    value={isDragging || isDebouncing ? sliderValue : currentTime}
                    onInput={(e) => {
                        setSliderValue(parseFloat(e.currentTarget.value));
                        handleSeek(parseFloat(e.currentTarget.value));
                    }}
                    onMouseDown={() => setIsDragging(true)}
                    onMouseUp={() => {
                        setIsDragging(false);
                        setIsDebouncing(true);
                        // Fallback timeout in case backend never catches up (e.g. paused)
                        setTimeout(() => setIsDebouncing(false), 500);
                    }}
                    className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                />

                {/* Hover Playhead Indicator (Optional/Advanced, simpler for now) */}
            </div>

            {/* Controls Row */}
            <div className="h-20 px-6 flex items-center justify-between">
                <div className="flex items-center gap-6 w-1/3">
                    <div className="flex flex-col min-w-0">
                        <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-0.5">Playing</span>
                        <div className="text-sm font-bold text-zinc-200 truncate pr-4" title={currentFile?.name}>
                            {currentFile ? currentFile.name : "Idle"}
                        </div>
                    </div>
                    <div className="flex flex-col border-l border-white/10 pl-6">
                        <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-0.5">Time</span>
                        <div className="text-sm font-mono font-medium text-brand-yellow tabular-nums">
                            {formatTime(currentTime)} <span className="text-zinc-600 font-normal mx-0.5">/</span> {formatTime(duration)}
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-4 no-drag">
                    <button
                        onClick={handleToggleAspect}
                        className={cn(
                            "p-2.5 rounded-xl transition-all relative group hover:scale-105 active:scale-95",
                            aspectMode === "Fit" ? "text-zinc-500 hover:text-zinc-300 hover:bg-white/5" : "text-brand-yellow bg-brand-yellow/10 border border-brand-yellow/20"
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

                    <button className="p-2.5 rounded-xl hover:bg-white/5 text-zinc-500 hover:text-zinc-300 transition-all">
                        <Subtitles className="w-5 h-5" />
                    </button>

                    <button
                        onClick={handleTogglePlayback}
                        className="w-14 h-14 flex items-center justify-center rounded-2xl bg-brand-yellow text-black hover:bg-brand-yellow-400 hover:scale-105 active:scale-95 shadow-lg shadow-brand-yellow/20 transition-all mx-2"
                    >
                        {isPlaying ? (
                            <Pause className="w-6 h-6 fill-current" />
                        ) : (
                            <Play className="w-6 h-6 fill-current ml-0.5" />
                        )}
                    </button>

                    {/* Volume Control */}
                    <div className="bg-white/5 rounded-xl p-1 border border-white/5">
                        <VolumeControl />
                    </div>

                    <button className="p-2.5 rounded-xl hover:bg-white/5 text-zinc-500 hover:text-zinc-300 transition-all">
                        <FastForward className="w-5 h-5" />
                    </button>
                </div>

                <div className="flex items-center justify-end gap-3 w-1/3">
                    <button
                        onClick={handleToggleQuality}
                        className={cn(
                            "flex items-center gap-2 px-4 py-2 rounded-xl border transition-all duration-300 no-drag",
                            qualityMode === "Native"
                                ? "bg-white/5 border-white/10 hover:border-white/20 text-zinc-300 hover:text-white"
                                : "bg-brand-yellow text-black border-brand-yellow shadow-lg shadow-brand-yellow/20"
                        )}
                        title={`Video Quality: ${qualityMode}`}
                    >
                        {qualityMode === "Native" && <ShieldCheck className="w-4 h-4" />}
                        {qualityMode === "Fast" && <Zap className="w-4 h-4" />}
                        {qualityMode === "Proxy" && <FastForward className="w-4 h-4" />}
                        <span className="text-xs font-bold tracking-tight uppercase">
                            {qualityMode === "Native" ? "High Fidelity" : qualityMode}
                        </span>
                    </button>
                </div>
            </div>
        </footer>
    );
};
