use std::fs::File;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Instant;
use sqlx::SqlitePool;
use crate::capture::CaptureFrame;
use crate::muxer::checkpoint::{checkpoint_session, SessionCheckpoint};

pub struct FragmentedMp4Writer {
    file: Option<File>,
    buffer: Vec<u8>,
    pub segment_duration_ms: u32,
    pub frame_count: u64,
    pub bytes_written: u64,
}

impl FragmentedMp4Writer {
    pub fn new(segment_duration_ms: u32) -> Self {
        Self {
            file: None,
            buffer: Vec::with_capacity(4 * 1024 * 1024),
            segment_duration_ms,
            frame_count: 0,
            bytes_written: 0,
        }
    }

    pub fn start_muxer_thread(mut self, rx: Receiver<CaptureFrame>, db_pool: SqlitePool, session_id: String) {
        thread::spawn(move || {
            println!("Muxer thread started, waiting for frames...");
            let mut last_checkpoint = Instant::now();
            
            while let Ok(frame) = rx.recv() {
                if let Err(e) = self.write_frame(&frame) {
                    eprintln!("Error writing frame: {}", e);
                    break;
                }
                
                // Checkpoint every 2 seconds
                if last_checkpoint.elapsed().as_secs() >= 2 {
                    let checkpoint = SessionCheckpoint {
                        session_id: session_id.clone(),
                        timestamp_ms: chrono::Utc::now().timestamp_millis(),
                        total_frames: self.frame_count as i64,
                        file_size_bytes: self.bytes_written as i64,
                        storage_status: "recording".to_string(),
                    };
                    
                    let pool = db_pool.clone();
                    // Spawn as a tokio task to not block the muxer if DB is slow
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = checkpoint_session(&pool, &checkpoint).await {
                            eprintln!("Failed to checkpoint session: {}", e);
                        }
                    });
                    
                    last_checkpoint = Instant::now();
                }
            }
            println!("Muxer thread stopped.");
        });
    }

    pub fn write_frame(&mut self, _frame: &CaptureFrame) -> io::Result<()> {
        self.frame_count += 1;
        // Pseudo logic: accumulate in buffer, flush to file
        if self.buffer.len() > 4 * 1024 * 1024 {
            self.flush_segment()?;
        }
        Ok(())
    }

    fn flush_segment(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            file.write_all(&self.buffer)?;
            file.sync_all()?;
            self.buffer.clear();
        }
        Ok(())
    }
}
