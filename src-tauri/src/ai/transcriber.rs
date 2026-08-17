use std::path::PathBuf;
use std::sync::mpsc::Receiver;
// use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams};
use sqlx::SqlitePool;

pub struct WhisperEngine {
    _model_path: PathBuf, // Mocked for now to avoid LLVM dependency on Windows
}

impl WhisperEngine {
    pub fn new(model_path: &PathBuf) -> Result<Self, String> {
        // let params = WhisperContextParameters::default();
        // let ctx = WhisperContext::new_with_params(&model_path.to_string_lossy(), params)
        //     .map_err(|e| format!("Failed to load model: {}", e))?;
        
        Ok(Self { _model_path: model_path.clone() })
    }

    pub fn start_transcription_thread(
        self,
        rx: Receiver<Vec<f32>>,
        db_pool: SqlitePool,
        session_id: String
    ) {
        std::thread::spawn(move || {
            // let mut state = self._context.create_state().expect("failed to create state");
            
            while let Ok(_audio_chunk) = rx.recv() {
                // MOCK INFERENCE for Windows build
                let text = "Mock transcribed text".to_string();
                let t0 = 0;
                let t1 = 5000;
                
                println!("[{}-{}] Transcript: {}", t0, t1, text);
                
                let pool = db_pool.clone();
                let sess_id = session_id.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = sqlx::query(
                        "INSERT INTO transcripts (session_id, start_time, end_time, text) VALUES (?1, ?2, ?3, ?4)"
                    )
                    .bind(sess_id)
                    .bind(t0)
                    .bind(t1)
                    .bind(text)
                    .execute(&pool)
                    .await;
                });
            }
        });
    }
}
