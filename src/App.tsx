import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { X, Minus, Square, Play, ShieldCheck, Zap } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

function App() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [isLowQuality, setIsLowQuality] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const unlisten = listen("tauri://drag-drop", async (event: any) => {
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        const path = paths[0];
        setFilePath(path);
        try {
          await invoke("open_video", { path });
        } catch (err) {
          console.error("Failed to open video:", err);
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleToggleQuality = async () => {
    const nextVal = !isLowQuality;
    setIsLowQuality(nextVal);
    await invoke("toggle_quality", { lowQuality: nextVal });
  };

  return (
    <div className="flex flex-col h-screen text-zinc-300 select-none overflow-hidden font-sans bg-transparent">
      {/* Custom Title Bar */}
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
          <button
            onClick={() => appWindow.minimize()}
            className="p-2 hover:bg-white/5 transition-colors"
          >
            <Minus className="w-4 h-4" />
          </button>
          <button
            onClick={() => appWindow.maximize()}
            className="p-2 hover:bg-white/5 transition-colors"
          >
            <Square className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => appWindow.close()}
            className="p-2 hover:bg-red-500/80 hover:text-white transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </header>

      {/* Main Content / Preview Area */}
      <main className="relative flex-1 flex items-center justify-center bg-pro-black/50 overflow-hidden">
        {!filePath ? (
          <div className="flex flex-col items-center gap-6 text-center animate-in fade-in zoom-in duration-500">
            <div className="w-24 h-24 rounded-3xl bg-brand-yellow/10 flex items-center justify-center border border-brand-yellow/20 shadow-2xl shadow-brand-yellow/5">
              <Play className="w-10 h-10 text-brand-yellow fill-current ml-1" />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Ready to Preview</h2>
              <p className="text-zinc-500 max-w-xs">Drag and drop any video file here to start the native WGPU playback engine.</p>
            </div>
          </div>
        ) : (
          <div className="absolute inset-0 flex items-center justify-center italic text-brand-yellow font-bold uppercase tracking-widest opacity-20 pointer-events-none">
            WGPU Native Layer Active
          </div>
        )}

        {/* Overlay Controls */}
        <div className="absolute bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-3 no-drag">
          <button
            onClick={handleToggleQuality}
            className={cn(
              "flex items-center gap-2 px-4 py-2 rounded-full border transition-all duration-300",
              isLowQuality
                ? "bg-brand-yellow text-pro-black border-brand-yellow shadow-lg shadow-brand-yellow/20"
                : "glass-panel border-white/10 hover:border-white/20"
            )}
          >
            {isLowQuality ? <Zap className="w-4 h-4" /> : <ShieldCheck className="w-4 h-4" />}
            <span className="text-sm font-bold tracking-tight uppercase">
              {isLowQuality ? "Performance Mode" : "Quality Mode"}
            </span>
          </button>
        </div>
      </main>

      {/* Footer Info */}
      <footer className="h-8 flex items-center justify-between px-4 text-[10px] uppercase tracking-widest text-zinc-600 glass-panel">
        <div className="truncate max-w-[200px]">{filePath ? filePath.split(/[\\/]/).pop() : "No File Loaded"}</div>
        <div className="flex items-center gap-4">
          <span>{isLowQuality ? "960x540 (1/4)" : "Native Res"}</span>
          <span className="text-brand-yellow/50">WGPU + FFmpeg Core</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
