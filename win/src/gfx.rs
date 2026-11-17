// GPU plumbing: one D3D11/D2D/DirectComposition device for the process, one flip-model swap chain
// per window. DirectComposition is what lets a swap chain keep per-pixel alpha on the desktop.
use windows::core::{w, Interface, Result, HSTRING};
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
    /// Distance from a text layout's top to its baseline: DrawText places text by its box.
    pub baseline: f32,
}

pub struct Surface {
    pub swap: IDXGISwapChain1,
    pub target: ID2D1Bitmap1,
    pub stat: ID2D1Bitmap1,  // static parts with their glows, baked once (draw::bake)
    pub layer: ID2D1Bitmap1, // the numbers, drawn here first so they can be glowed
    pub glow: ID2D1Effect,   // the numbers' glow: shadow of `layer`
    pub size: D2D_SIZE_U,
    pub dpi: f32,
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
        // Space Mono straight from fonts/, through a private font collection (no install needed)
        let dwrite: IDWriteFactory5 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        let path = HSTRING::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../fonts/SpaceMono-Regular.ttf"));
        let builder = dwrite.CreateFontSetBuilder()?;
        builder.AddFontFile(&dwrite.CreateFontFileReference(&path, None)?)?;
        let fonts = dwrite.CreateFontCollectionFromFontSet(&builder.CreateFontSet()?)?;
        let format = |size| {
            dwrite.CreateTextFormat(
                w!("Space Mono"),
                &fonts,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("en-us"),
            )
        };
        let (title, value) = (format(12.0)?, format(12.0)?);
        value.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
        let layout = dwrite.CreateTextLayout(&[b'0' as u16], &title, 100.0, 100.0)?;
        let (mut lines, mut count) = ([DWRITE_LINE_METRICS::default()], 0);
        layout.GetLineMetrics(Some(&mut lines), &mut count)?;
        let baseline = lines[0].baseline;
        Ok(Self { dxgi, swap_factory, factory, dc, dcomp, title, value, baseline })
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
        let layer = self.bitmap(size, dpi)?;
        let stat = self.bitmap(size, dpi)?;
        // canvas shadowBlur 8 = standard deviation 4; color as in the reference
        let glow = self.shadow(&layer, 4.0, 0.6)?;
        let comp_target = self.dcomp.CreateTargetForHwnd(hwnd, true)?;
        let visual = self.dcomp.CreateVisual()?;
        visual.SetContent(&swap)?;
        comp_target.SetRoot(&visual)?;
        self.dcomp.Commit()?;
        Ok(Surface { swap, target, stat, layer, glow, size, dpi, _comp: (comp_target, visual) })
    }

    pub unsafe fn bitmap(&self, size: D2D_SIZE_U, dpi: f32) -> Result<ID2D1Bitmap1> {
        self.dc.CreateBitmap(size, None, 0, &props(D2D1_BITMAP_OPTIONS_TARGET, dpi))
    }

    /// Canvas-style shadow of `input`: its alpha blurred (sigma = shadowBlur / 2), tinted accent.
    pub unsafe fn shadow(&self, input: &ID2D1Bitmap1, sigma: f32, alpha: f32) -> Result<ID2D1Effect> {
        let e = self.dc.CreateEffect(&CLSID_D2D1Shadow)?;
        e.SetInput(0, input, true);
        let color = [80.0f32 / 255.0, 1.0, 120.0 / 255.0, alpha];
        e.SetValue(D2D1_SHADOW_PROP_COLOR.0 as u32, D2D1_PROPERTY_TYPE_VECTOR4, bytes(&color))?;
        e.SetValue(D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32, D2D1_PROPERTY_TYPE_FLOAT, bytes(&[sigma]))?;
        Ok(e)
    }
}

fn bytes(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast(), std::mem::size_of_val(v)) }
}
