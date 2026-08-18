use windows::Win32::Graphics::Direct3D11::*; fn test(dev: &ID3D11Device) { let mut desc = D3D11_TEXTURE2D_DESC::default(); unsafe { dev.CreateTexture2D(&desc, None, None); } }
