use std::sync::Mutex;
use tauri::State;
use crate::capture::windows::WindowsCaptureEngine;

pub struct AppState {
    pub capture_engine: Mutex<WindowsCaptureEngine>,
}

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    println!("Recording started!");
    let mut engine = state.capture_engine.lock().unwrap();
    engine.start().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    println!("Recording stopped!");
    let mut engine = state.capture_engine.lock().unwrap();
    engine.stop();
    Ok(())
}
