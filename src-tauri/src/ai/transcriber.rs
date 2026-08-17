use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams};
use sqlx::SqlitePool;

pub struct WhisperEngine {
    _context: WhisperContext,
}

impl WhisperEngine {
    pub fn new(model_path: &PathBuf) -> Result<Self, String> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(&model_path.to_string_lossy(), params)
            .map_err(|e| format!("Failed to load model: {}", e))?;
        
        Ok(Self { _context: ctx })
    }

    pub fn start_transcription_thread(
        self,
        rx: Receiver<Vec<f32>>,
        db_pool: SqlitePool,
        session_id: String
    ) {
        std::thread::spawn(move || {
            let mut state = self._context.create_state().expect("failed to create state");
            
            while let Ok(audio_chunk) = rx.recv() {
                let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
                params.set_language(Some("en"));
                params.set_print_progress(false);
                params.set_print_special(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);
                
                if let Err(e) = state.full(params, &audio_chunk[..]) {
                    eprintln!("Whisper inference failed: {}", e);
                    continue;
                }
                
                let num_segments = state.full_n_segments().unwrap_or(0);
                for i in 0..num_segments {
                    if let Ok(text) = state.full_get_segment_text(i) {
                        let t0 = state.full_get_segment_t0(i).unwrap_or(0);
                        let t1 = state.full_get_segment_t1(i).unwrap_or(0);
                        
                        println!("[{}-{}] Transcript: {}", t0, t1, text);
                        
                        // Insert into database asynchronously
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
                }
            }
        });
    }
}
