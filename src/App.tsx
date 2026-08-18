import { useState } from "react";
import { FloatingHUD } from "./components/FloatingHUD";
import { MediaLibrary } from "./components/MediaLibrary";
import CameraPiP from "./components/CameraPiP";
import { PreRecordingConfig } from "./components/PreRecordingConfig";
import "./App.css";

function App() {
  const [showLibrary, setShowLibrary] = useState(false);
  const [appState, setAppState] = useState<"config" | "recording">("config");

  return (
    <div className="w-screen h-screen bg-transparent overflow-hidden pointer-events-none relative flex items-center justify-center">
      
      {appState === "config" ? (
        <div className="pointer-events-auto">
          <PreRecordingConfig onStart={() => setAppState("recording")} />
        </div>
      ) : (
        <>
          {/* Recording Border Indicator */}
          <div className="absolute inset-0 border-4 border-red-500/80 animate-pulse pointer-events-none rounded-lg" style={{ boxShadow: "inset 0 0 20px rgba(239, 68, 68, 0.5)" }} />
          
          <div className="pointer-events-auto absolute top-4 left-4 z-50">
            <FloatingHUD onOpenLibrary={() => setShowLibrary(true)} onStop={() => setAppState("config")} />
          </div>
          
          <div className="pointer-events-auto z-50">
            <CameraPiP />
          </div>
        </>
      )}

      {showLibrary && (
        <div className="pointer-events-auto absolute inset-0 z-50 flex items-center justify-center bg-black/50 p-8">
          <div className="bg-[#0f0f13] w-full max-w-4xl h-[80vh] rounded-2xl shadow-2xl overflow-hidden flex flex-col relative border border-slate-800">
            <button 
              onClick={() => setShowLibrary(false)}
              className="absolute top-4 right-4 z-10 p-2 bg-slate-800 hover:bg-slate-700 rounded-full text-white transition-colors"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </button>
            <MediaLibrary />
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
