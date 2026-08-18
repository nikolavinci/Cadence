use windows::Win32::Media::MediaFoundation::*;

use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::core::{GUID, Interface};

const CLSID_CMSH264EncoderMFT: GUID = GUID::from_values(0x6ca50344, 0x051a, 0x4ded, [0x97, 0x79, 0xa4, 0x33, 0x05, 0x16, 0x5e, 0x35]);

pub struct MfH264Encoder {
    initialized: bool,
    mft: Option<IMFTransform>,
}

unsafe impl Send for MfH264Encoder {}
unsafe impl Sync for MfH264Encoder {}

impl MfH264Encoder {
    pub fn new(width: u32, height: u32) -> windows::core::Result<Self> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            MFStartup(MF_VERSION, MFSTARTUP_NOSOCKET)?;

            println!("Instantiating H.264 MFT for {}x{}...", width, height);
            let mft: IMFTransform = CoCreateInstance(&CLSID_CMSH264EncoderMFT, None, CLSCTX_INPROC_SERVER)?;

            let out_type = MFCreateMediaType()?;
            out_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            out_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
            out_type.SetUINT32(&MF_MT_AVG_BITRATE, 5_000_000)?;
            out_type.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | (height as u64))?;
            out_type.SetUINT64(&MF_MT_FRAME_RATE, (30u64 << 32) | 1u64)?;
            out_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            mft.SetOutputType(0, &out_type, 0)?;

            let in_type = MFCreateMediaType()?;
            in_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            in_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)?;
            in_type.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | (height as u64))?;
            in_type.SetUINT64(&MF_MT_FRAME_RATE, (30u64 << 32) | 1u64)?;
            in_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            mft.SetInputType(0, &in_type, 0)?;

            mft.ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0)?;
            mft.ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0)?;

            Ok(Self {
                initialized: true,
                mft: Some(mft),
            })
        }
    }

    pub fn encode_frame(&mut self, raw_data: &[u8]) -> Vec<u8> {
        if raw_data.is_empty() {
            return Vec::new();
        }
        
        let mut compressed_data = Vec::new();
        
        unsafe {
            if let Some(mft) = &self.mft {
                if let Ok(buffer) = MFCreateMemoryBuffer(raw_data.len() as u32) {
                    let mut ptr = std::ptr::null_mut();
                    if buffer.Lock(&mut ptr, None, None).is_ok() {
                        std::ptr::copy_nonoverlapping(raw_data.as_ptr(), ptr, raw_data.len());
                        let _ = buffer.Unlock();
                        let _ = buffer.SetCurrentLength(raw_data.len() as u32);
                        
                        if let Ok(sample) = MFCreateSample() {
                            let _ = sample.AddBuffer(&buffer);
                            let _ = mft.ProcessInput(0, &sample, 0);
                        }
                    }
                }
                
                loop {
                    let mut output_buffer = MFT_OUTPUT_DATA_BUFFER::default();
                    let mut status = 0;
                    
                    if mft.ProcessOutput(0, std::slice::from_mut(&mut output_buffer), &mut status).is_err() {
                        break;
                    }
                    
                    if let Some(sample) = &*output_buffer.pSample {
                        if let Ok(mb) = sample.ConvertToContiguousBuffer() {
                            let mut ptr = std::ptr::null_mut();
                            let mut len = 0;
                            if mb.Lock(&mut ptr, None, Some(&mut len)).is_ok() {
                                let slice = std::slice::from_raw_parts(ptr, len as usize);
                                compressed_data.extend_from_slice(slice);
                                let _ = mb.Unlock();
                            }
                        }
                    }
                }
            }
        }
        
        compressed_data
    }
}

impl Drop for MfH264Encoder {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                let _ = MFShutdown();
            }
        }
    }
}
