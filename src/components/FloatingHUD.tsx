import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AudioLevels {
  mic: number;
  system: number;
}

export const FloatingHUD: React.FC = () => {
  const [isRecording, setIsRecording] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [audioLevels, setAudioLevels] = useState<AudioLevels>({ mic: 0, system: 0 });

  // Simulate audio levels for now
  useEffect(() => {
    if (isRecording) {
      const interval = setInterval(() => {
        setElapsed((e) => e + 1);
        setAudioLevels({
          mic: Math.random() * 0.8 + 0.1,
          system: Math.random() * 0.5,
        });
      }, 1000);
      return () => clearInterval(interval);
    }
  }, [isRecording]);

  const handleStartStop = useCallback(async () => {
    if (isRecording) {
      await invoke("stop_recording");
      setIsRecording(false);
    } else {
      await invoke("start_recording");
      setElapsed(0);
      setIsRecording(true);
    }
  }, [isRecording]);

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-50 bg-black/80 rounded-lg px-6 py-3 text-white shadow-2xl backdrop-blur-md flex flex-col items-center">
      {/* Timer */}
      <div className="text-xl font-mono mb-2 tracking-wider">
        {formatTime(elapsed)}
      </div>

      {/* Audio Level Meters */}
      <div className="flex gap-4 w-full mb-4">
        <div className="flex flex-col flex-1 items-center gap-1">
          <span className="text-xs text-gray-400">MIC</span>
          <meter value={audioLevels.mic} max="1.0" className="w-full h-2 rounded-full overflow-hidden [&::-webkit-meter-bar]:bg-gray-700 [&::-webkit-meter-optimum-value]:bg-green-500" />
        </div>
        <div className="flex flex-col flex-1 items-center gap-1">
          <span className="text-xs text-gray-400">SYS</span>
          <meter value={audioLevels.system} max="1.0" className="w-full h-2 rounded-full overflow-hidden [&::-webkit-meter-bar]:bg-gray-700 [&::-webkit-meter-optimum-value]:bg-blue-500" />
        </div>
      </div>

      {/* Transport Controls */}
      <div className="flex gap-4">
        <button
          onClick={handleStartStop}
          className={`px-6 py-2 rounded font-semibold transition-colors ${
            isRecording ? "bg-red-600 hover:bg-red-700" : "bg-blue-600 hover:bg-blue-700"
          }`}
        >
          {isRecording ? "Stop Capture" : "Start Capture"}
        </button>
      </div>
    </div>
  );
};
