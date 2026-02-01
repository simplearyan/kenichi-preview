import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/library/Sidebar";
import { PreviewArea } from "./components/player/PreviewArea";
import { ControlBar } from "./components/player/ControlBar";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useFileProcessing } from "./hooks/useFileProcessing";
import { useStore } from "./store/useStore";

function App() {
  const setIsRendererReady = useStore((state) => state.setIsRendererReady);

  // Initialize Global Listeners
  useTauriEvents();
  useFileProcessing();

  // Initial Renderer Setup
  useEffect(() => {
    const init = async () => {
      try {
        await invoke("init_renderer");
        setIsRendererReady(true);
      } catch (err) {
        console.error("Failed to init renderer:", err);
      }
    };
    init();
  }, []);

  return (
    <div className="flex flex-col h-screen text-zinc-300 select-none overflow-hidden font-sans bg-transparent">
      <TitleBar />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar />
        <PreviewArea />
      </div>
      <ControlBar />
    </div>
  );
}

export default App;
