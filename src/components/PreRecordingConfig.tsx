import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export const PreRecordingConfig: React.FC<{ onStart: () => void }> = ({ onStart }) => {
  const [micEnabled, setMicEnabled] = useState(true);
  const [sysAudioEnabled, setSysAudioEnabled] = useState(true);
  const [cameraEnabled, setCameraEnabled] = useState(true);
  const [isDownloadingModel, setIsDownloadingModel] = useState(false);
  const [modelStatus, setModelStatus] = useState("AI Transcription (Whisper)");

  const handleDownloadModel = async () => {
    setIsDownloadingModel(true);
    setModelStatus("Downloading ggml-tiny.en.bin (~140MB)...");
    try {
      const path = await invoke<string>("download_whisper_model");
      setModelStatus(`Model ready: ${path}`);
    } catch (err) {
      setModelStatus(`Download failed: ${err}`);
    } finally {
      setIsDownloadingModel(false);
    }
  };

  const handleStart = async () => {
    await invoke("start_recording");
    onStart();
  };

  return (
    <div className="bg-[#0f0f13] text-white rounded-2xl shadow-2xl border border-slate-800 w-[480px] flex flex-col font-sans overflow-hidden">
      {/* Custom Title Bar for Dragging */}
      <div 
        data-tauri-drag-region 
        className="h-10 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 select-none cursor-grab active:cursor-grabbing"
      >
        <div data-tauri-drag-region className="text-xs font-semibold text-slate-400 tracking-wider">CORE-RECORDER</div>
        <div className="flex gap-2">
          <button onClick={() => invoke('plugin:window|minimize')} className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400" />
          <button onClick={() => invoke('plugin:window|close')} className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400" />
        </div>
      </div>

      <div className="p-8 flex flex-col gap-6">
        <div>
          <h1 className="text-3xl font-bold mb-1">Setup Recording</h1>
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

        <div className="p-4 rounded-lg bg-blue-900/20 border border-blue-900/50 flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <span className="font-medium text-blue-200 text-sm">{modelStatus}</span>
            <button 
              onClick={handleDownloadModel}
              disabled={isDownloadingModel || modelStatus.startsWith("Model ready")}
              className="text-xs px-3 py-1 bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-400 rounded transition-colors"
            >
              {isDownloadingModel ? "Downloading..." : (modelStatus.startsWith("Model ready") ? "Downloaded" : "Download Model")}
            </button>
          </div>
        </div>
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
