use std::fs::File;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::thread;
use crate::capture::CaptureFrame;

pub struct FragmentedMp4Writer {
    file: Option<File>,
    buffer: Vec<u8>,
    pub segment_duration_ms: u32,
    pub frame_count: u64,
}

impl FragmentedMp4Writer {
    pub fn new(segment_duration_ms: u32) -> Self {
        Self {
            file: None,
            buffer: Vec::with_capacity(4 * 1024 * 1024),
            segment_duration_ms,
            frame_count: 0,
        }
    }

    pub fn start_muxer_thread(mut self, rx: Receiver<CaptureFrame>) {
        thread::spawn(move || {
            println!("Muxer thread started, waiting for frames...");
            while let Ok(frame) = rx.recv() {
                if let Err(e) = self.write_frame(&frame) {
                    eprintln!("Error writing frame: {}", e);
                    break;
                }
            }
            println!("Muxer thread stopped.");
        });
    }

    pub fn write_frame(&mut self, _frame: &CaptureFrame) -> io::Result<()> {
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
