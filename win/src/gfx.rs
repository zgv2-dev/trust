// GPU plumbing: one D3D11/D2D/DirectComposition device for the process, one flip-model swap chain
// per window. DirectComposition is what lets a swap chain keep per-pixel alpha on the desktop.
use windows::core::{w, Interface, Result};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

pub struct Gpu {
    dxgi: IDXGIDevice,
    swap_factory: IDXGIFactory2,
    pub factory: ID2D1Factory1,
    pub dc: ID2D1DeviceContext,
    dcomp: IDCompositionDevice,
    pub title: IDWriteTextFormat,
    pub value: IDWriteTextFormat,
}

pub struct Surface {
    pub swap: IDXGISwapChain1,
    pub target: ID2D1Bitmap1,
    pub layer: ID2D1Bitmap1, // foreground drawn here first, so it can be glowed and composited
    pub glow: ID2D1Effect,
    _comp: (IDCompositionTarget, IDCompositionVisual),
}

fn props(options: D2D1_BITMAP_OPTIONS, dpi: f32) -> D2D1_BITMAP_PROPERTIES1 {
    D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
        dpiX: dpi,
        dpiY: dpi,
        bitmapOptions: options,
        ..Default::default()
    }
}

impl Gpu {
    pub unsafe fn new(dpi: f32) -> Result<Self> {
        let mut d3d = None;
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT, // required for Direct2D on D3D11
            None,
            D3D11_SDK_VERSION,
            Some(&mut d3d),
            None,
            None,
        )?;
        let dxgi: IDXGIDevice = d3d.unwrap().cast()?;
        let swap_factory: IDXGIFactory2 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
        let factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let dc = factory.CreateDevice(&dxgi)?.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        dc.SetDpi(dpi, dpi);
        let dcomp: IDCompositionDevice = DCompositionCreateDevice(&dxgi)?;
        let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        let format = |size| {
            dwrite.CreateTextFormat(
                w!("Consolas"),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("en-us"),
            )
        };
        let (title, value) = (format(12.0)?, format(12.0)?);
        value.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
        Ok(Self { dxgi, swap_factory, factory, dc, dcomp, title, value })
    }

    pub unsafe fn surface(&self, hwnd: HWND, width: u32, height: u32, dpi: f32) -> Result<Surface> {
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            ..Default::default()
        };
        let swap = self.swap_factory.CreateSwapChainForComposition(&self.dxgi, &desc, None)?;
        let back: IDXGISurface = swap.GetBuffer(0)?;
        let target = self.dc.CreateBitmapFromDxgiSurface(
            &back,
            Some(&props(D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW, dpi)),
        )?;
        let size = D2D_SIZE_U { width, height };
        let layer = self.dc.CreateBitmap(size, None, 0, &props(D2D1_BITMAP_OPTIONS_TARGET, dpi))?;
        let glow = self.dc.CreateEffect(&CLSID_D2D1Shadow)?;
        glow.SetInput(0, &layer, true);
        let accent = [80.0f32 / 255.0, 1.0, 120.0 / 255.0, 0.8];
        glow.SetValue(D2D1_SHADOW_PROP_COLOR.0 as u32, D2D1_PROPERTY_TYPE_VECTOR4, bytes(&accent))?;
        glow.SetValue(D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32, D2D1_PROPERTY_TYPE_FLOAT, bytes(&[3.0f32]))?;
        let comp_target = self.dcomp.CreateTargetForHwnd(hwnd, true)?;
        let visual = self.dcomp.CreateVisual()?;
        visual.SetContent(&swap)?;
        comp_target.SetRoot(&visual)?;
        self.dcomp.Commit()?;
        Ok(Surface { swap, target, layer, glow, _comp: (comp_target, visual) })
    }
}

fn bytes(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast(), std::mem::size_of_val(v)) }
}
