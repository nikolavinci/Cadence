pub mod windows;

use std::sync::Arc;

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
