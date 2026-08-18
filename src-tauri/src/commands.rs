use std::sync::Mutex;
use tauri::State;
use crate::capture::windows::WindowsCaptureEngine;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use reqwest::Client;
use tauri::Manager;

pub struct AppState {
    pub capture_engine: Mutex<WindowsCaptureEngine>,
    pub audio_engine: Mutex<crate::capture::audio::AudioCaptureEngine>,
    pub whisper_tx: Mutex<Option<std::sync::mpsc::Sender<Vec<f32>>>>,
}

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    println!("Recording started!");
    
    let mut engine = state.capture_engine.lock().unwrap();
    engine.start().map_err(|e| e.to_string())?;
    
    let mut audio_engine = state.audio_engine.lock().unwrap();
    if let Some(tx) = state.whisper_tx.lock().unwrap().clone() {
        audio_engine.start(tx)?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    println!("Recording stopped!");
    let mut engine = state.capture_engine.lock().unwrap();
    engine.stop();
    
    let mut audio_engine = state.audio_engine.lock().unwrap();
    audio_engine.stop();
    Ok(())
}

#[tauri::command]
pub async fn download_whisper_model(app_handle: tauri::AppHandle) -> Result<String, String> {
    let model_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin";
    
    let app_dir = app_handle.path().app_data_dir().map_err(|_| "Failed to get app data dir".to_string())?;
    let model_path = app_dir.join("ggml-tiny.en.bin");
    
    if model_path.exists() {
        return Ok(model_path.to_string_lossy().to_string());
    }

    let client = Client::new();
    let response = client.get(model_url).send().await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    
    let mut file = File::create(&model_path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    
    Ok(model_path.to_string_lossy().to_string())
}
