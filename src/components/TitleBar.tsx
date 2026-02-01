import { Minus, Square, Play, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

export const TitleBar = () => {
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
        </header>
    );
};
