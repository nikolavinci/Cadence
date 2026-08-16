pub mod capture;
pub mod muxer;
pub mod commands;

use std::sync::Mutex;
use capture::windows::WindowsCaptureEngine;
use capture::CapturePipeline;
use muxer::fmp4::FragmentedMp4Writer;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pipeline = CapturePipeline::new();
    let mut engine = WindowsCaptureEngine::new();
    engine.set_sender(pipeline.tx);
    
    // Start muxer in background thread
    let muxer = FragmentedMp4Writer::new(2000);
    muxer.start_muxer_thread(pipeline.rx);

    tauri::Builder::default()
        .manage(commands::AppState {
            capture_engine: Mutex::new(engine),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::start_recording,
            commands::stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
