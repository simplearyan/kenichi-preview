import { Maximize2, Subtitles, Play, Pause, FastForward, ShieldCheck, Zap, Trash2 } from "lucide-react";
import { useRef, useState, useEffect } from "react";
import { useStore } from "../../store/useStore";
import { usePlayback } from "../../hooks/usePlayback";
import { useTrimActions } from "../../hooks/useTrimActions";
import { formatTime } from "../../utils/format";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { VolumeControl } from "./VolumeControl";
import { useInterpolatedTime } from "../../hooks/useInterpolatedTime";

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

    // Direct DOM Refs for performance (No Re-renders)
    const progressBarRef = useRef<HTMLDivElement>(null);
    const timeDisplayRef = useRef<HTMLDivElement>(null);
    const sliderRef = useRef<HTMLInputElement>(null);

    // Callbacks for the interpolation loop
    const onTimeUpdate = (time: number) => {
        // 1. Update Progress Bar
        if (progressBarRef.current) {
            const pct = (time / (duration || 1)) * 100;
            progressBarRef.current.style.width = `${pct}%`;
        }
        // 2. Update Slider (if not dragging)
        if (sliderRef.current && !isDragging) {
            // We don't update input.value programmatically to avoid interfering with interaction
            // instead we rely on visual bar. But for "knob" position we might need it? 
            // Actually, for a pure CSS bar, we don't need the input knob to move perfectly if it's invisible.
            // BUT, if we want the knob to be at the right place when user clicks?
            // Yes, we should update it.
            sliderRef.current.value = time.toString();
        }
        // 3. Update Text
        if (timeDisplayRef.current) {
            timeDisplayRef.current.innerText = `${formatTime(time)} / ${formatTime(duration)}`;
        }
    };

    // Smooth interpolated time hook (Updates refs directly)
    useInterpolatedTime(currentTime, isPlaying, onTimeUpdate);

    // Trim Actions
    const { trimStart, trimEnd, setMarkIn, setMarkOut, clearMarks } = useTrimActions();

    // Keyboard Shortcuts for Trim
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Ignore if input focused
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

            switch (e.key.toLowerCase()) {
                case 'i':
                    setMarkIn();
                    break;
                case 'o':
                    setMarkOut();
                    break;
                case 'delete':
                case 'backspace':
                    // Verify modifier? Or is backspace dangerous? Let's require Shift+Delete for Clear
                    if (e.shiftKey) clearMarks();
                    break;
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [setMarkIn, setMarkOut, clearMarks]);

    // Local state for dragging (still needed for Interaction)
    const [isDragging, setIsDragging] = useState(false);
    // sliderValue state is removed, we read directly from event

    const currentFile = currentIndex !== null ? playlist[currentIndex] : null;

    // Calculate Trim visual percentages
    const startPct = trimStart !== undefined ? (trimStart / (duration || 1)) * 100 : 0;
    const endPct = trimEnd !== undefined ? (trimEnd / (duration || 1)) * 100 : 100;
    const widthPct = Math.max(0, endPct - startPct);
    const hasTrim = trimStart !== undefined || trimEnd !== undefined;

    const isImage = currentFile?.type === "Image";

    return (
        <div className="flex flex-col z-50 relative">
            {/* Centered Action Bar (Floating above ControlBar) */}
            <div className={cn(
                "absolute -top-14 left-1/2 -translate-x-1/2 bg-zinc-900/90 backdrop-blur-xl border border-white/10 rounded-xl px-2 py-1 flex items-center gap-1 transition-all duration-300 shadow-xl",
                hasTrim ? "opacity-100 translate-y-0" : "opacity-0 translate-y-2 pointer-events-none"
            )}>
                {/* Set In Button */}
                <button onClick={setMarkIn} className="p-1.5 hover:bg-white/10 rounded-lg text-zinc-400 hover:text-white transition-colors" title="Set In Point (I)">
                    <span className="font-bold text-xs">[</span>
                </button>

                <div className="flex items-center gap-2 px-2">
                    <span className="text-[10px] font-mono font-medium text-pink-500">
                        {trimStart !== undefined ? formatTime(trimStart) : "START"}
                    </span>
                    <span className="text-zinc-600 text-[10px]">•</span>
                    <span className="text-[10px] font-mono font-medium text-pink-500">
                        {trimEnd !== undefined ? formatTime(trimEnd) : "END"}
                    </span>
                    <span className="bg-white/10 rounded px-1.5 py-0.5 text-[9px] font-mono text-zinc-300">
                        {trimStart !== undefined && trimEnd !== undefined ? formatTime(trimEnd - trimStart) : "--:--"}
                    </span>
                </div>

                {/* Set Out Button */}
                <button onClick={setMarkOut} className="p-1.5 hover:bg-white/10 rounded-lg text-zinc-400 hover:text-white transition-colors" title="Set Out Point (O)">
                    <span className="font-bold text-xs">]</span>
                </button>

                <div className="w-px h-4 bg-white/10 mx-1" />

                <button onClick={clearMarks} className="p-1.5 hover:bg-white/10 rounded-lg text-zinc-400 hover:text-red-400 transition-colors" title="Clear In/Out Points (Shift+Del)">
                    <Trash2 className="w-4 h-4" />
                </button>
            </div>

            <footer className="glass-panel border-t border-white/5 flex flex-col relative bg-zinc-950/80 backdrop-blur-xl overflow-hidden">
                {/* Progress Bar - Only for non-images */}
                {!isImage && (
                    <div className="w-full h-1.5 relative group cursor-pointer bg-white/5">
                        {/* Progress Fill */}
                        <div
                            ref={progressBarRef}
                            className="absolute top-0 left-0 bottom-0 bg-brand-yellow/80 group-hover:bg-brand-yellow transition-all duration-75 ease-linear"
                            style={{ width: `${(currentTime / (duration || 1)) * 100}%` }}
                        />

                        {/* Selected Trim Range */}
                        {hasTrim && (
                            <div
                                className="absolute top-0 bottom-0 bg-pink-500/30 pointer-events-none"
                                style={{
                                    left: `${startPct}%`,
                                    width: `${widthPct}%`,
                                    zIndex: 5
                                }}
                            >
                                <div className="absolute left-0 top-0 bottom-0 w-0.5 bg-pink-500 shadow-[0_0_10px_rgba(236,72,153,0.5)]" />
                                <div className="absolute right-0 top-0 bottom-0 w-0.5 bg-pink-500 shadow-[0_0_10px_rgba(236,72,153,0.5)]" />
                            </div>
                        )}

                        <input
                            ref={sliderRef}
                            type="range"
                            min="0"
                            max={duration || 100}
                            step="0.01"
                            defaultValue={currentTime}
                            onInput={(e) => {
                                const val = parseFloat(e.currentTarget.value);
                                handleSeek(val);
                                onTimeUpdate(val);
                            }}
                            onMouseDown={() => setIsDragging(true)}
                            onMouseUp={() => setIsDragging(false)}
                            className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                        />
                    </div>
                )}

                {/* Controls Row */}
                <div className="h-20 px-6 flex items-center justify-between gap-8">
                    {/* Left Section: File Info & Time */}
                    <div className="flex items-center gap-6 flex-1 min-w-0">
                        <div className="flex flex-col min-w-0 max-w-[240px]">
                            <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1">
                                {isImage ? "Displaying" : "Playing"}
                            </span>
                            <div className="text-xs font-bold text-zinc-200 truncate pr-4" title={currentFile?.name}>
                                {currentFile ? currentFile.name : "Idle"}
                            </div>
                        </div>

                        {!isImage && (
                            <div className="flex flex-col border-l border-white/10 pl-6 shrink-0">
                                <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1">Time</span>
                                <div ref={timeDisplayRef} className="text-xs font-mono font-medium text-brand-yellow tabular-nums tracking-tight">
                                    {formatTime(currentTime)} / {formatTime(duration)}
                                </div>
                            </div>
                        )}
                    </div>

                    {/* Center Section: Playback Controls */}
                    <div className="flex items-center gap-2 no-drag shrink-0">
                        <div className="flex items-center bg-white/5 rounded-xl border border-white/5 p-1">
                            <button
                                onClick={handleToggleAspect}
                                className={cn(
                                    "p-2 rounded-lg transition-all relative group hover:bg-white/5",
                                    aspectMode === "Fit" ? "text-zinc-500" : "text-brand-yellow bg-brand-yellow/10"
                                )}
                                title={`Aspect Ratio: ${aspectMode}`}
                            >
                                <Maximize2 className="w-4 h-4" />
                            </button>
                            <button className="p-2 rounded-lg hover:bg-white/5 text-zinc-500 transition-all">
                                <Subtitles className="w-4 h-4" />
                            </button>
                        </div>

                        {!isImage ? (
                            <button
                                onClick={handleTogglePlayback}
                                className="w-12 h-12 flex items-center justify-center rounded-xl bg-brand-yellow text-black hover:bg-brand-yellow-400 hover:scale-105 active:scale-95 shadow-lg shadow-brand-yellow/20 transition-all mx-1"
                            >
                                {isPlaying ? (
                                    <Pause className="w-5 h-5 fill-current" />
                                ) : (
                                    <Play className="w-5 h-5 fill-current ml-0.5" />
                                )}
                            </button>
                        ) : (
                            <div className="w-12 h-12" /> // Spacer for symmetry
                        )}

                        <div className="flex items-center bg-white/5 rounded-xl border border-white/5 p-1">
                            <div className="px-1 border-r border-white/10 mr-1">
                                <VolumeControl />
                            </div>
                            <button className="p-2 rounded-lg hover:bg-white/5 text-zinc-500 transition-all">
                                <FastForward className="w-4 h-4" />
                            </button>
                        </div>
                    </div>

                    {/* Right Section: Trim & Quality */}
                    <div className="flex items-center justify-end gap-3 flex-1">
                        {!isImage && (
                            <div className="flex items-center gap-1 bg-white/5 rounded-xl border border-white/5 p-1">
                                <button
                                    onClick={setMarkIn}
                                    className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-white/10 text-zinc-400 hover:text-white transition-colors"
                                    title="Set In Point (I)"
                                >
                                    <span className="font-bold text-xs">[</span>
                                </button>
                                <button
                                    onClick={setMarkOut}
                                    className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-white/10 text-zinc-400 hover:text-white transition-colors"
                                    title="Set Out Point (O)"
                                >
                                    <span className="font-bold text-xs">]</span>
                                </button>
                            </div>
                        )}

                        <button
                            onClick={handleToggleQuality}
                            className={cn(
                                "flex items-center gap-2 px-3 py-1.5 rounded-xl border transition-all duration-300 no-drag shrink-0",
                                qualityMode === "Native"
                                    ? "bg-white/5 border-white/10 hover:border-white/20 text-zinc-400 hover:text-zinc-200"
                                    : "bg-brand-yellow text-black border-brand-yellow shadow-lg shadow-brand-yellow/20"
                            )}
                            title={`Video Quality: ${qualityMode}`}
                        >
                            {qualityMode === "Native" && <ShieldCheck className="w-3.5 h-3.5" />}
                            {qualityMode === "Fast" && <Zap className="w-3.5 h-3.5" />}
                            <span className="text-[10px] font-bold tracking-tight uppercase">
                                {qualityMode === "Native" ? "Hi-Fi" : qualityMode}
                            </span>
                        </button>
                    </div>
                </div>
            </footer>
        </div>
    );
};
