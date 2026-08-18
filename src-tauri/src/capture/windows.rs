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
    Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
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
            
            let mut encoder = MfH264Encoder::new()?;
            println!("H.264 Media Foundation Encoder Initialized");

            let mut desc = D3D11_TEXTURE2D_DESC::default();
            desc.Width = item_size.Width as u32;
            desc.Height = item_size.Height as u32;
            desc.MipLevels = 1;
            desc.ArraySize = 1;
            desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
            desc.SampleDesc.Count = 1;
            desc.Usage = D3D11_USAGE_STAGING;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;

            let mut staging_texture: Option<ID3D11Texture2D> = None;
            _d3d_device.CreateTexture2D(&desc, None, Some(&mut staging_texture))?;
            let staging_texture = staging_texture.unwrap();
            let d3d_context_clone = d3d_context.as_ref().unwrap().clone();

            // Setup FrameArrived Callback
            if let Some(tx) = self.tx.clone() {
                let encoder_arc = Arc::new(std::sync::Mutex::new(encoder));
                
                let handler = windows::Foundation::TypedEventHandler::<Direct3D11CaptureFramePool, windows::core::IInspectable>::new(
                    move |pool, _| {
                        if let Some(pool_ref) = &*pool {
                            if let Ok(frame) = pool_ref.TryGetNextFrame() {
                                let surface = frame.Surface()?;
                                
                                unsafe {
                                    let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
                                    let source_texture: ID3D11Texture2D = access.GetInterface()?;
                                    
                                    // 1. Copy GPU texture to Staging texture
                                    d3d_context_clone.CopyResource(&staging_texture, &source_texture);
                                    
                                    // 2. Map staging texture to CPU memory
                                    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                                    d3d_context_clone.Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
                                    
                                    // 3. Read the BGRA bytes
                                    let pitch = mapped.RowPitch as usize;
                                    let height = item_size.Height as usize;
                                    let width = item_size.Width as usize;
                                    
                                    // Allocate buffer for contiguous pixel data (ignoring pitch padding for now)
                                    let mut raw_pixels = Vec::with_capacity(width * height * 4);
                                    let ptr = mapped.pData as *const u8;
                                    
                                    for y in 0..height {
                                        let row_slice = std::slice::from_raw_parts(ptr.add(y * pitch), width * 4);
                                        raw_pixels.extend_from_slice(row_slice);
                                    }
                                    
                                    d3d_context_clone.Unmap(&staging_texture, 0);
                                    
                                    println!("✅ Copied GPU frame to CPU. Bytes extracted: {}", raw_pixels.len());

                                    // For our structural architecture, we simulate the compression:
                                    if let Ok(mut enc) = encoder_arc.lock() {
                                        let compressed_bytes = enc.encode_frame(&raw_pixels);
                                        
                                        let capture_frame = CaptureFrame {
                                            stream_id: StreamId::Screen,
                                            pts_ns: frame.SystemRelativeTime()?.Duration as u64 * 100, // 100ns ticks to ns
                                            sequence: 0,
                                            data: Arc::from(compressed_bytes.into_boxed_slice()),
                                            metadata: FrameMetadata {
                                                sample_rate: None,
                                                channels: None,
                                                width: Some(item_size.Width as u32),
                                                height: Some(item_size.Height as u32),
                                            },
                                        };
                                        
                                        let _ = tx.send(capture_frame);
                                    }
                                }
                            }
                        }
                        Ok(())
                    }
                );
                frame_pool.FrameArrived(&handler)?;
            }

            session.StartCapture()?;

            self.frame_pool = Some(frame_pool);
            self.capture_session = Some(session);

            println!("Successfully started capturing Primary Monitor: {}x{}", item_size.Width, item_size.Height);
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
