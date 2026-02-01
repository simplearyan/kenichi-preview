import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TitleBar } from "./components/TitleBar";
import { Sidebar } from "./components/Sidebar";
import { PreviewArea } from "./components/PreviewArea";
import { ControlBar } from "./components/ControlBar";
import { useTauriEvents } from "./hooks/useTauriEvents";

function App() {
  // Initialize Global Listeners
  useTauriEvents();

  // Initial Renderer Setup
  useEffect(() => {
    const init = async () => {
      try {
        await invoke("init_renderer");
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
