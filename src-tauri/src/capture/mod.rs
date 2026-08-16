use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, channel};

pub mod windows;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamId {
    Screen,
    Camera,
    Mic,
    Loopback,
}

#[derive(Debug, Clone)]
pub struct FrameMetadata {
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CaptureFrame {
    pub stream_id: StreamId,
    pub pts_ns: u64, // Monotonic nanoseconds
    pub sequence: u64,
    pub data: Arc<[u8]>,
    pub metadata: FrameMetadata,
}

pub struct CapturePipeline {
    pub tx: Sender<CaptureFrame>,
    pub rx: Receiver<CaptureFrame>,
}

impl CapturePipeline {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }
}
