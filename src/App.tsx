import { FloatingHUD } from "./components/FloatingHUD";
import { MediaLibrary } from "./components/MediaLibrary";
import "./App.css";

function App() {
  return (
    <div className="min-h-screen bg-gray-900 flex flex-col items-center justify-center p-8 text-white relative">
      <header className="mb-12 text-center">
        <h1 className="text-4xl font-bold mb-4">Core-Recorder v2.0</h1>
        <p className="text-gray-400 max-w-lg">
          Zero-cloud, local-first desktop recording application for screencast creators and technical coaches.
        </p>
      </header>

      <main className="w-full max-w-5xl grid grid-cols-1 md:grid-cols-2 gap-8 mb-24">
        {/* Media Library component */}
        <div className="min-h-[300px]">
          <MediaLibrary />
        </div>
        <div className="bg-gray-800 rounded-xl p-8 border border-gray-700 flex flex-col items-start justify-center min-h-[300px]">
          <h2 className="text-xl font-semibold mb-2">Session Info</h2>
          <ul className="text-sm text-gray-400 space-y-2">
            <li>• Codec: H.264 / AAC</li>
            <li>• FPS: 60</li>
            <li>• Storage: Fragmented MP4</li>
          </ul>
        </div>
      </main>

      {/* The Floating HUD overlay */}
      <FloatingHUD />
    </div>
  );
}

export default App;
