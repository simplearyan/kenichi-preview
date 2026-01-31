import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Minus, Square, Play, Pause, ShieldCheck, Zap, Plus, Film, Image as ImageIcon, FastForward, Subtitles, ListVideo, X } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

function App() {
  const [playlist, setPlaylist] = useState<string[]>([]);
  const [currentIndex, setCurrentIndex] = useState<number | null>(null);
  const [isLowQuality, setIsLowQuality] = useState(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const appWindow = getCurrentWindow();
  const mainRef = useRef<HTMLElement>(null);

  const currentFile = currentIndex !== null ? playlist[currentIndex] : null;

  const updateViewport = async () => {
    if (!mainRef.current) return;
    const rect = mainRef.current.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;

    // Scale to physical pixels for WGPU
    await invoke("update_viewport", {
      x: rect.left * dpr,
      y: rect.top * dpr,
      width: rect.width * dpr,
      height: rect.height * dpr,
    });
  };

  const handleOpenFile = async (path: string) => {
    try {
      await invoke("open_video", { path });
      setIsPlaying(true);
      // Wait a tick for layout if needed, then update viewport
      setTimeout(updateViewport, 100);
    } catch (err) {
      console.error("Failed to open video:", err);
    }
  };

  const handleImport = async () => {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Media",
          extensions: ["mp4", "mkv", "avi", "mov", "webm", "jpg", "png", "webp"],
        },
      ],
    });

    if (selected && Array.isArray(selected)) {
      setPlaylist(prev => [...prev, ...selected]);
      if (currentIndex === null) {
        setCurrentIndex(playlist.length);
        handleOpenFile(selected[0]);
      }
    } else if (selected) {
      setPlaylist(prev => [...prev, selected]);
      if (currentIndex === null) {
        setCurrentIndex(playlist.length);
        handleOpenFile(selected);
      }
    }
  };

  const selectMedia = (index: number) => {
    setCurrentIndex(index);
    handleOpenFile(playlist[index]);
  };

  useEffect(() => {
    const unlisten = listen("tauri://drag-drop", async (event: any) => {
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        setPlaylist(prev => [...prev, ...paths]);
        if (currentIndex === null) {
          setCurrentIndex(playlist.length);
          handleOpenFile(paths[0]);
        }
      }
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === "Space") {
        e.preventDefault();
        handleTogglePlayback();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    // Viewport updates
    updateViewport();
    window.addEventListener("resize", updateViewport);

    return () => {
      unlisten.then((fn) => fn());
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", updateViewport);
    };
  }, [currentIndex, playlist, isPlaying]);

  const handleToggleQuality = async () => {
    const nextVal = !isLowQuality;
    setIsLowQuality(nextVal);
    await invoke("toggle_quality", { lowQuality: nextVal });
  };

  const handleTogglePlayback = async () => {
    if (currentIndex === null) return;
    try {
      const playing = await invoke<boolean>("toggle_playback");
      setIsPlaying(playing);
    } catch (err) {
      console.error("Failed to toggle playback:", err);
    }
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

      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar: Playlist Management */}
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

        {/* Main Preview Area */}
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
            <>
              {/* Central Playback HUD */}
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
            </>
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
      </div>

      {/* Modern Control Bar */}
      <footer className="h-20 glass-panel border-t border-white/5 px-6 flex items-center justify-between z-50">
        <div className="flex items-center gap-6 w-1/3">
          <div className="flex flex-col">
            <span className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-1">Active Preview</span>
            <div className="text-sm font-bold text-white truncate max-w-[200px]">
              {currentFile ? currentFile.split(/[\\/]/).pop() : "Idle"}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-4 no-drag">
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

          <button className="p-2.5 rounded-xl hover:bg-white/5 text-zinc-500 transition-all">
            <FastForward className="w-5 h-5" />
          </button>
        </div>

        <div className="flex items-center justify-end gap-3 w-1/3">
          <button
            onClick={handleToggleQuality}
            className={cn(
              "flex items-center gap-2 px-4 py-2 rounded-xl border transition-all duration-300 no-drag",
              isLowQuality
                ? "bg-brand-yellow text-pro-black border-brand-yellow shadow-lg shadow-brand-yellow/20"
                : "bg-white/5 border-white/10 hover:border-white/20 text-white"
            )}
          >
            {isLowQuality ? <Zap className="w-4 h-4" /> : <ShieldCheck className="w-4 h-4" />}
            <span className="text-xs font-bold tracking-tight uppercase">
              {isLowQuality ? "Performance" : "Native High"}
            </span>
          </button>
        </div>
      </footer>
    </div>
  );
}

export default App;
