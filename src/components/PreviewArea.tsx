import { useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Play, Pause } from "lucide-react";
import { useStore } from "../store/useStore";
import { usePlayback } from "../hooks/usePlayback";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const PreviewArea = () => {
    const mainRef = useRef<HTMLElement>(null);
    const isRendererReady = useStore((state) => state.isRendererReady);
    const currentIndex = useStore((state) => state.currentIndex);
    const isPlaying = useStore((state) => state.isPlaying);

    const { handleImport, handleTogglePlayback } = usePlayback();

    const updateViewport = async () => {
        if (!mainRef.current || !isRendererReady) return;
        const rect = mainRef.current.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;

        console.log(`[Viewport] Syncing to rect:`, rect);
        await invoke("update_viewport", {
            x: Math.floor(rect.left * dpr),
            y: Math.floor(rect.top * dpr),
            width: Math.floor(rect.width * dpr),
            height: Math.floor(rect.height * dpr),
        });
    };

    useEffect(() => {
        if (!mainRef.current) return;

        const observer = new ResizeObserver(() => {
            updateViewport();
        });

        observer.observe(mainRef.current);
        updateViewport();

        return () => observer.disconnect();
    }, [isRendererReady, currentIndex]);

    return (
        <main
            ref={mainRef}
            onDoubleClick={handleImport}
            onClick={handleTogglePlayback}
            className="flex-1 relative flex items-center justify-center bg-transparent overflow-hidden cursor-pointer group"
        >
            {currentIndex === null ? (
                <div className="flex flex-col items-center gap-6 text-center animate-in fade-in zoom-in duration-500">
                    <div className="w-24 h-24 rounded-3xl bg-brand-yellow/10 flex items-center justify-center border border-brand-yellow/20 shadow-2xl shadow-brand-yellow/5">
                        <Plus className="w-10 h-10 text-brand-yellow ml-1" />
                    </div>
                    <div className="bg-pro-black/40 backdrop-blur-md px-6 py-4 rounded-2xl border border-white/5">
                        <h2 className="text-2xl font-bold text-white mb-2 italic">Kenichi<span className="text-brand-yellow">Preview</span></h2>
                        <p className="text-zinc-500 max-w-xs text-sm">Select or drop video files to begin the native WGPU preview.</p>
                    </div>
                </div>
            ) : (
                <div className={cn(
                    "w-24 h-24 rounded-full bg-black/40 backdrop-blur-xl flex items-center justify-center border border-white/10 transition-all duration-300 shadow-2xl",
                    !isPlaying ? "opacity-100 scale-100" : "opacity-0 scale-90 group-hover:opacity-100 group-hover:scale-100"
                )}>
                    {isPlaying ? (
                        <Pause className="w-10 h-10 text-white fill-current" />
                    ) : (
                        <Play className="w-10 h-10 text-brand-yellow fill-current ml-1" />
                    )}
                </div>
            )}

            {/* Overlay Status */}
            <div className="absolute top-6 left-6 pointer-events-none animate-in fade-in slide-in-from-left-4 duration-700">
                <div className="flex flex-col gap-1">
                    <span className="text-[10px] font-bold text-brand-yellow/50 uppercase tracking-[0.2em] mb-1">
                        Renderer Status
                    </span>
                    <div className="bg-pro-black/60 backdrop-blur-md px-3 py-1.5 rounded-lg border border-white/5 flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                        <span className="text-[11px] font-medium text-white tracking-wide">
                            NATIVE WGPU CORE ACTIVE
                        </span>
                    </div>
                </div>
            </div>
        </main>
    );
};
