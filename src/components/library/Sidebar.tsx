import { ListVideo, Plus, Film, Music, Image as ImageIcon } from "lucide-react";
import { useStore } from "../../store/useStore";
import { usePlayback } from "../../hooks/usePlayback";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const Sidebar = () => {
    const { playlist, currentIndex } = useStore();
    const { handleImport, selectMedia } = usePlayback();

    const formatSize = (bytes?: number) => {
        if (!bytes) return "";
        const mb = bytes / (1024 * 1024);
        return mb > 1000 ? `${(mb / 1024).toFixed(1)} GB` : `${Math.round(mb)} MB`;
    };

    const getResolutionBadge = (width?: number, height?: number) => {
        if (!width || !height) return null;
        const min = Math.min(width, height);
        if (min >= 2160) return "4K";
        if (min >= 1440) return "2K";
        if (min >= 1080) return "1080p";
        if (min >= 720) return "720p";
        return "SD";
    };

    return (
        <aside className="w-80 flex flex-col glass-panel border-r border-white/5 z-40 bg-zinc-900/50 backdrop-blur-xl">
            <div className="p-4 flex items-center justify-between border-b border-white/5 bg-zinc-900/20">
                <div className="flex items-center gap-2">
                    <ListVideo className="w-4 h-4 text-brand-yellow" />
                    <h2 className="text-xs font-bold uppercase tracking-wider text-white">Media Library</h2>
                    <span className="text-[10px] text-zinc-500 font-mono px-1.5 py-0.5 bg-white/5 rounded-full">
                        {playlist.length}
                    </span>
                </div>
                <button
                    onClick={handleImport}
                    className="p-1.5 rounded-lg bg-brand-yellow/10 text-brand-yellow hover:bg-brand-yellow/20 transition-all border border-brand-yellow/20 hover:scale-105 active:scale-95"
                >
                    <Plus className="w-4 h-4" />
                </button>
            </div>

            <div className="flex-1 overflow-y-auto p-3 space-y-2 custom-scrollbar">
                {playlist.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-center p-4">
                        <div className="w-16 h-16 rounded-2xl bg-white/5 flex items-center justify-center mb-4 border border-white/5">
                            <Film className="w-8 h-8 text-zinc-600" />
                        </div>
                        <h3 className="text-sm font-medium text-zinc-300 mb-1">No Media</h3>
                        <p className="text-[10px] text-zinc-500 leading-relaxed max-w-[150px]">
                            Drag and drop files here to start your project
                        </p>
                    </div>
                ) : (
                    playlist.map((item, idx) => {
                        const isActive = currentIndex === idx;
                        const formatDuration = (s: number) => {
                            if (!s) return "0:00";
                            const mins = Math.floor(s / 60);
                            const secs = Math.floor(s % 60);
                            return `${mins}:${secs.toString().padStart(2, "0")}`;
                        };

                        const badge = getResolutionBadge(item.width, item.height);

                        return (
                            <button
                                key={item.path + idx}
                                onClick={() => selectMedia(idx)}
                                className={cn(
                                    "w-full flex items-start gap-3 p-3 rounded-xl transition-all group text-left relative overflow-hidden",
                                    isActive
                                        ? "bg-brand-yellow/5 border-brand-yellow/20"
                                        : "hover:bg-white/5 border-transparent hover:border-white/10"
                                )}
                            >
                                {/* Active Indicator Bar */}
                                {isActive && (
                                    <div className="absolute left-0 top-0 bottom-0 w-1 bg-brand-yellow" />
                                )}

                                {/* Thumbnail Area */}
                                <div className={cn(
                                    "w-24 aspect-video rounded-lg flex items-center justify-center shrink-0 transition-all overflow-hidden bg-zinc-900 shadow-lg border border-white/5 relative group-hover:border-white/10",
                                    isActive && "ring-1 ring-brand-yellow/20"
                                )}>
                                    {item.thumbnail ? (
                                        <img
                                            src={item.thumbnail}
                                            alt={item.name}
                                            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                                        />
                                    ) : item.processing ? (
                                        <div className="w-4 h-4 rounded-full border-2 border-brand-yellow/30 border-t-brand-yellow animate-spin" />
                                    ) : item.type === 'Audio' ? (
                                        <div className="flex flex-col items-center gap-1">
                                            <Music className="w-5 h-5 text-zinc-600" />
                                            <div className="flex gap-0.5">
                                                {[1, 2, 3, 2, 1].map((h, i) => (
                                                    <div key={i} className="w-0.5 bg-zinc-700 rounded-full" style={{ height: h * 4 }} />
                                                ))}
                                            </div>
                                        </div>
                                    ) : (
                                        <ImageIcon className="w-5 h-5 text-zinc-600" />
                                    )}

                                    {/* Duration Overlay on Thumbnail */}
                                    {item.duration ? (
                                        <div className="absolute bottom-1 right-1 px-1 py-0.5 bg-black/80 rounded flex items-center gap-1 backdrop-blur-sm border border-white/10">
                                            <span className="text-[9px] font-bold text-white font-mono leading-none">
                                                {formatDuration(item.duration)}
                                            </span>
                                        </div>
                                    ) : null}
                                </div>

                                {/* Content Area */}
                                <div className="flex-1 min-w-0 flex flex-col justify-between h-full py-0.5">
                                    <div>
                                        <div className="flex items-start justify-between gap-2 mb-1">
                                            <div className={cn(
                                                "text-xs font-bold truncate leading-tight",
                                                isActive ? "text-brand-yellow" : "text-zinc-300 group-hover:text-white"
                                            )} title={item.name}>
                                                {item.name}
                                            </div>
                                        </div>

                                        <div className="flex flex-wrap items-center gap-1.5">
                                            {/* Resolution Badge */}
                                            {badge && (
                                                <span className={cn(
                                                    "text-[9px] font-bold px-1 rounded border",
                                                    badge === "4K" ? "text-brand-yellow border-brand-yellow/30 bg-brand-yellow/10" :
                                                        badge === "1080p" ? "text-blue-400 border-blue-400/30 bg-blue-400/10" :
                                                            "text-zinc-500 border-zinc-700 bg-zinc-800"
                                                )}>
                                                    {badge}
                                                </span>
                                            )}

                                            {/* File Type Badge */}
                                            <span className="text-[9px] font-bold text-zinc-500 px-1 rounded border border-zinc-700 bg-zinc-800 uppercase">
                                                {item.container || item.path.split('.').pop() || 'FILE'}
                                            </span>
                                        </div>
                                    </div>

                                    {/* Bottom Meta Row */}
                                    <div className="flex items-center gap-2 mt-2 pt-2 border-t border-white/5">
                                        {item.size && (
                                            <span className="text-[10px] text-zinc-500 font-mono">
                                                {formatSize(item.size)}
                                            </span>
                                        )}
                                        {item.sampleRate && (
                                            <>
                                                <div className="w-0.5 h-0.5 rounded-full bg-zinc-600" />
                                                <span className="text-[10px] text-zinc-500 font-mono">
                                                    {item.sampleRate}Hz
                                                </span>
                                            </>
                                        )}
                                    </div>
                                </div>
                            </button>
                        );
                    })
                )}
            </div>
        </aside>
    );
};
