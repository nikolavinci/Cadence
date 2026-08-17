pub mod capture;
pub mod muxer;
pub mod commands;
pub mod ai;

use std::sync::Mutex;

use capture::windows::WindowsCaptureEngine;
use capture::CapturePipeline;
use muxer::fmp4::FragmentedMp4Writer;
use crate::muxer::checkpoint::{init_db, recover_orphaned_sessions};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&app_dir).unwrap();
            let db_path = app_dir.join("session_state.sqlite");
            
            tauri::async_runtime::block_on(async {
                let pool = init_db(&db_path).await.expect("Failed to initialize SQLite DB");
                
                // Phase 3.5: Attempt to recover any abruptly terminated sessions
                if let Err(e) = recover_orphaned_sessions(&pool).await {
                    eprintln!("Failed to recover orphaned sessions: {}", e);
                }
                
                app.manage(pool);
            });

            let pipeline = CapturePipeline::new();
            let mut engine = WindowsCaptureEngine::new();
            engine.set_sender(pipeline.tx);
            
            let muxer = FragmentedMp4Writer::new(2000);
            let session_id = format!("session_{}", chrono::Utc::now().timestamp_millis());
            let db_pool = app.state::<sqlx::SqlitePool>().inner().clone();
            muxer.start_muxer_thread(pipeline.rx, db_pool, session_id);
            
            app.manage(commands::AppState {
                capture_engine: Mutex::new(engine),
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::start_recording,
            commands::stop_recording,
            commands::download_whisper_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
