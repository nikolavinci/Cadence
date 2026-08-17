import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export const PreRecordingConfig: React.FC<{ onStart: () => void }> = ({ onStart }) => {
  const [micEnabled, setMicEnabled] = useState(true);
  const [sysAudioEnabled, setSysAudioEnabled] = useState(true);
  const [cameraEnabled, setCameraEnabled] = useState(true);

  const handleStart = async () => {
    await invoke("start_recording");
    onStart();
  };

  return (
    <div className="bg-[#0f0f13] text-white p-8 rounded-2xl shadow-2xl border border-slate-800 w-[480px] flex flex-col gap-6 font-sans">
      <div>
        <h1 className="text-3xl font-bold mb-1">Core-Recorder v2.0</h1>
        <p className="text-slate-400 text-sm">Configure your capture session</p>
      </div>

      <div className="flex flex-col gap-4">
        <label className="flex items-center justify-between cursor-pointer p-3 rounded-lg bg-slate-800/50 hover:bg-slate-800 transition-colors">
          <span className="font-medium text-slate-200">Microphone</span>
          <input type="checkbox" checked={micEnabled} onChange={(e) => setMicEnabled(e.target.checked)} className="w-5 h-5 accent-blue-500" />
        </label>
        
        <label className="flex items-center justify-between cursor-pointer p-3 rounded-lg bg-slate-800/50 hover:bg-slate-800 transition-colors">
          <span className="font-medium text-slate-200">System Audio</span>
          <input type="checkbox" checked={sysAudioEnabled} onChange={(e) => setSysAudioEnabled(e.target.checked)} className="w-5 h-5 accent-blue-500" />
        </label>
        
        <label className="flex items-center justify-between cursor-pointer p-3 rounded-lg bg-slate-800/50 hover:bg-slate-800 transition-colors">
          <span className="font-medium text-slate-200">Camera PiP</span>
          <input type="checkbox" checked={cameraEnabled} onChange={(e) => setCameraEnabled(e.target.checked)} className="w-5 h-5 accent-blue-500" />
        </label>
      </div>

      <button 
        onClick={handleStart}
        className="w-full py-4 mt-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-xl transition-all shadow-[0_0_15px_rgba(37,99,235,0.4)] hover:shadow-[0_0_25px_rgba(37,99,235,0.6)]"
      >
        Start Recording
      </button>
    </div>
  );
};
