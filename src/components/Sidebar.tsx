import { ListVideo, Plus, Image as ImageIcon, Film } from "lucide-react";
import { useStore } from "../store/useStore";
import { usePlayback } from "../hooks/usePlayback";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const Sidebar = () => {
    const { playlist, currentIndex } = useStore();
    const { handleImport, selectMedia } = usePlayback();

    return (
        <aside className="w-72 flex flex-col glass-panel border-r border-white/5 z-40">
            <div className="p-4 flex items-center justify-between border-b border-white/5">
                <div className="flex items-center gap-2">
                    <ListVideo className="w-4 h-4 text-brand-yellow" />
                    <h2 className="text-xs font-bold uppercase tracking-wider text-white">Media Library</h2>
                </div>
                <button
                    onClick={handleImport}
                    className="p-1.5 rounded-lg bg-brand-yellow/10 text-brand-yellow hover:bg-brand-yellow/20 transition-all border border-brand-yellow/20"
                >
                    <Plus className="w-4 h-4" />
                </button>
            </div>

            <div className="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                {playlist.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-center p-4">
                        <ImageIcon className="w-10 h-10 text-zinc-700 mb-3" />
                        <p className="text-[10px] text-zinc-500 leading-relaxed uppercase tracking-widest">
                            Drop files to start
                        </p>
                    </div>
                ) : (
                    playlist.map((path, idx) => {
                        const name = path.split(/[\\/]/).pop();
                        const isActive = currentIndex === idx;
                        return (
                            <button
                                key={path + idx}
                                onClick={() => selectMedia(idx)}
                                className={cn(
                                    "w-full flex items-center gap-3 p-3 rounded-xl transition-all group text-left",
                                    isActive
                                        ? "bg-brand-yellow/10 border border-brand-yellow/20 shadow-lg"
                                        : "hover:bg-white/5 border border-transparent"
                                )}
                            >
                                <div className={cn(
                                    "w-8 h-8 rounded-lg flex items-center justify-center shrink-0 transition-all",
                                    isActive ? "bg-brand-yellow text-pro-black" : "bg-zinc-800 text-zinc-400 group-hover:bg-zinc-700"
                                )}>
                                    {path.match(/\.(jpg|jpeg|png|webp|gif)$/i) ? (
                                        <ImageIcon className="w-4 h-4" />
                                    ) : (
                                        <Film className="w-4 h-4" />
                                    )}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className={cn(
                                        "text-xs font-medium truncate mb-0.5",
                                        isActive ? "text-white" : "text-zinc-400 group-hover:text-zinc-300"
                                    )}>
                                        {name}
                                    </div>
                                    <div className="text-[9px] text-zinc-600 truncate uppercase tracking-tighter">
                                        Local File System
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
