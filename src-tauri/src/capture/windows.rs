use windows::{
    core::*,
    Graphics::Capture::*,
    Graphics::DirectX::*,
    Graphics::DirectX::Direct3D11::*,
    Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::*,
    Win32::Graphics::Gdi::*,
    Win32::Foundation::*,
    Win32::System::WinRT::*,
    Win32::System::WinRT::Direct3D11::*,
    Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop,
};
use std::sync::mpsc::Sender;
use crate::capture::{CaptureFrame, StreamId, FrameMetadata};
use crate::capture::encoder::MfH264Encoder;
use std::sync::Arc;

pub struct WindowsCaptureEngine {
    capture_session: Option<GraphicsCaptureSession>,
    frame_pool: Option<Direct3D11CaptureFramePool>,
    encoder: Option<MfH264Encoder>,
    tx: Option<Sender<CaptureFrame>>,
}

impl WindowsCaptureEngine {
    pub fn new() -> Self {
        Self {
            capture_session: None,
            frame_pool: None,
            encoder: None,
            tx: None,
        }
    }

    pub fn set_sender(&mut self, tx: Sender<CaptureFrame>) {
        self.tx = Some(tx);
    }

    pub fn start(&mut self) -> Result<()> {
        println!("Initializing Direct3D 11 device for capture...");
        
        unsafe {
            let mut d3d_device: Option<ID3D11Device> = None;
            let mut d3d_context: Option<ID3D11DeviceContext> = None;
            
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                Some(&mut d3d_context),
            )?;

            let _d3d_device = d3d_device.as_ref().unwrap();
            let dxgi_device: IDXGIDevice = _d3d_device.cast()?;
            let device_inspectable = CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)?;
            let winrt_device: IDirect3DDevice = device_inspectable.cast()?;

            // 1. Get Primary Monitor
            let mut monitor_handle: HMONITOR = HMONITOR::default();
            unsafe extern "system" fn enum_monitor(hmonitor: HMONITOR, _: HDC, _: *mut RECT, lparam: LPARAM) -> BOOL {
                let ptr = lparam.0 as *mut HMONITOR;
                unsafe { *ptr = hmonitor };
                BOOL::from(false) // Stop enumerating after the first one (primary)
            }

            EnumDisplayMonitors(None, None, Some(enum_monitor), LPARAM(&mut monitor_handle as *mut _ as isize));

            // 2. Create GraphicsCaptureItem
            let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
            let capture_item: GraphicsCaptureItem = interop.CreateForMonitor(monitor_handle)?;
            let item_size = capture_item.Size()?;

            // 3. Setup Direct3D11CaptureFramePool
            let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
                &winrt_device,
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                1,
                item_size,
            )?;

            // 4. Start Capture Session
            let session = frame_pool.CreateCaptureSession(&capture_item)?;
            session.StartCapture()?;

            let mut encoder = MfH264Encoder::new()?;
            println!("H.264 Media Foundation Encoder Initialized");

            self.frame_pool = Some(frame_pool);
            self.capture_session = Some(session);

            println!("Successfully started capturing Primary Monitor: {}x{}", item_size.Width, item_size.Height);
            
            // Simulate sending an initial frame for the walkthrough
            if let Some(tx) = &self.tx {
                let compressed_bytes = encoder.encode_frame(&[0; 4]);
                let dummy_frame = CaptureFrame {
                    stream_id: StreamId::Screen,
                    pts_ns: 0,
                    sequence: 0,
                    data: Arc::from(compressed_bytes.into_boxed_slice()),
                    metadata: FrameMetadata {
                        sample_rate: None,
                        channels: None,
                        width: Some(item_size.Width as u32),
                        height: Some(item_size.Height as u32),
                    },
                };
                let _ = tx.send(dummy_frame);
            }
            
            self.encoder = Some(encoder);
        }
        
        Ok(())
    }
            
    pub fn stop(&mut self) {
        if let Some(session) = self.capture_session.take() {
            let _ = session.Close();
        }
        if let Some(pool) = self.frame_pool.take() {
            let _ = pool.Close();
        }
        println!("Windows capture stopped.");
    }
}
