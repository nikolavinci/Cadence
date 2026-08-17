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

    pub fn write_frame(&mut self, frame: &CaptureFrame) -> io::Result<()> {
        self.frame_count += 1;
        
        // If this is the first frame, write the initialization segment (ftyp + moov)
        if self.frame_count == 1 {
            self.write_initialization_segment()?;
        }

        // Accumulate pseudo frame data
        self.buffer.extend_from_slice(&frame.data);

        // Every chunk (simulated by buffer size), write moof and mdat
        if self.buffer.len() > 1024 * 1024 {
            self.flush_segment()?;
        }
        Ok(())
    }

    fn write_box_header(buf: &mut Vec<u8>, box_type: &[u8; 4], payload_size: u32) {
        let size = 8 + payload_size;
        buf.extend_from_slice(&size.to_be_bytes());
        buf.extend_from_slice(box_type);
    }

    fn write_initialization_segment(&mut self) -> io::Result<()> {
        let mut ftyp_payload = Vec::new();
        ftyp_payload.extend_from_slice(b"iso5"); // Major brand
        ftyp_payload.extend_from_slice(&512u32.to_be_bytes()); // Minor version
        ftyp_payload.extend_from_slice(b"iso5"); // Compatible brands
        ftyp_payload.extend_from_slice(b"mp41");

        Self::write_box_header(&mut self.buffer, b"ftyp", ftyp_payload.len() as u32);
        self.buffer.extend_from_slice(&ftyp_payload);

        // Dummy moov for proof-of-concept
        let moov_payload = b"dummy_moov_payload";
        Self::write_box_header(&mut self.buffer, b"moov", moov_payload.len() as u32);
        self.buffer.extend_from_slice(moov_payload);

        Ok(())
    }

    fn flush_segment(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            let mut segment_buf = Vec::new();
            
            // Write moof
            let moof_payload = b"dummy_moof_payload";
            Self::write_box_header(&mut segment_buf, b"moof", moof_payload.len() as u32);
            segment_buf.extend_from_slice(moof_payload);
            
            // Write mdat
            Self::write_box_header(&mut segment_buf, b"mdat", self.buffer.len() as u32);
            segment_buf.extend_from_slice(&self.buffer);

            file.write_all(&segment_buf)?;
            file.sync_all()?;
            self.bytes_written += segment_buf.len() as u64;
            self.buffer.clear();
        }
        Ok(())
    }
}
