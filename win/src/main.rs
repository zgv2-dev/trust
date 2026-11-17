// Windows-native variant: Win32 windows without redirection surfaces, drawn with Direct2D into
// DirectComposition swap chains. BENCH_WINDOWS widgets, one window each, or all in one overlay
// window over the primary monitor when BENCH_MODE=overlay. Redraws once per DWM frame.
#[cfg(windows)]
mod draw;
#[cfg(windows)]
mod gfx;

#[cfg(not(windows))]
fn main() {
    eprintln!("bench-win only runs on Windows");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    unsafe { app::run() }
}

#[cfg(windows)]
mod app {
    use crate::draw::{self, Brushes, H, W};
    use crate::gfx::{Gpu, Surface};
    use std::time::Instant;
    use windows::core::{w, Result};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Dwm::DwmFlush;
    use windows::Win32::Graphics::Dxgi::DXGI_PRESENT;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::HiDpi::{
        GetDpiForSystem, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
            return LRESULT(0);
        }
        DefWindowProcW(hwnd, msg, wp, lp)
    }

    fn origin(i: usize) -> (f32, f32) {
        (40.0 + 260.0 * (i % 4) as f32, 40.0 + 140.0 * (i / 4) as f32)
    }

    pub unsafe fn run() -> Result<()> {
        let n: usize = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
        let overlay = std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay");
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
        let dpi = GetDpiForSystem() as f32;
        let scale = dpi / 96.0;
        let instance = GetModuleHandleW(None)?.into();
        let class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            lpszClassName: w!("bench-win"),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        RegisterClassW(&class);
        let gpu = Gpu::new(dpi)?;
        let brushes = Brushes::new(&gpu.dc)?;

        // (window, panel origins inside it in DIPs)
        let mut layout: Vec<((i32, i32, i32, i32), Vec<(f32, f32)>)> = vec![];
        if overlay {
            let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
            layout.push(((0, 0, sw, sh), (0..n).map(origin).collect()));
        } else {
            for i in 0..n {
                let (x, y) = origin(i);
                let px = |v: f32| (v * scale).round() as i32;
                layout.push(((px(x), px(y), px(W), px(H)), vec![(0.0, 0.0)]));
            }
        }
        let mut windows: Vec<(Surface, Vec<(f32, f32)>)> = vec![];
        for ((x, y, w, h), origins) in layout {
            let hwnd = CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("bench-win"),
                w!("bench-win"),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                w,
                h,
                None,
                None,
                Some(instance),
                None,
            )?;
            if overlay {
                clip::to_rects(hwnd.0, &clip::panels(n, scale as f64));
            }
            windows.push((gpu.surface(hwnd, w as u32, h as u32, dpi)?, origins));
        }

        // BENCH_FREEZE: draw that instant forever, for screenshots
        let freeze: Option<f32> = std::env::var("BENCH_FREEZE").ok().and_then(|t| t.parse().ok());
        let start = Instant::now();
        let mut first = true;
        let (mut frames, mut last) = (0u32, Instant::now());
        let mut msg = MSG::default();
        loop {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return Ok(());
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let t = freeze.unwrap_or_else(|| start.elapsed().as_secs_f32());
            for (surface, origins) in &windows {
                draw::frame(&gpu, &brushes, surface, origins, t)?;
                surface.swap.Present(0, DXGI_PRESENT(0)).ok()?;
            }
            // one wait per frame for all windows; Present(1) per swap chain would stack N vsync waits
            DwmFlush()?;
            if std::mem::take(&mut first) {
                println!("first-frame");
            }
            frames += 1;
            if last.elapsed().as_secs_f32() >= 1.0 {
                println!("fps {:.0}", frames as f32 / last.elapsed().as_secs_f32());
                (frames, last) = (0, Instant::now());
            }
        }
    }
}
