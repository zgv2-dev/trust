// egui variant: BENCH_WINDOWS widgets, one transparent window each, or all in one click-through
// overlay window when BENCH_MODE=overlay. Renders with wgpu, or OpenGL when BENCH_RENDERER=glow.
mod draw;

use eframe::egui::{self, pos2, Pos2, ViewportBuilder, ViewportId};
use halo::{H, MARGIN, W};
use std::time::Instant;

/// Frames during which window positions are re-sent: macOS can ignore or shift the first ones
/// (borderless windows created ~30 pt low, an overlay pushed below the menu bar while it grows).
const PLACE_FRAMES: u32 = 30;

struct Bench {
    windows: usize,
    start: Instant,
    frames: u32,
    total: u32,
    last: Instant,
    overlay: bool,
    freeze: Option<f32>, // BENCH_FREEZE: draw that instant forever, for screenshots
    glows: Option<draw::Glows>,
    #[cfg(windows)]
    clipped: bool,
}

fn pos(i: usize) -> Pos2 {
    pos2(40.0 + 260.0 * (i % 4) as f32, 40.0 + 140.0 * (i / 4) as f32)
}

/// Window i: the panel plus MARGIN on every side for its outer glow.
fn viewport(i: usize) -> ViewportBuilder {
    let at = pos(i) - egui::vec2(MARGIN, MARGIN);
    ViewportBuilder::default()
        .with_title(format!("egui {i}"))
        .with_inner_size([W + 2.0 * MARGIN, H + 2.0 * MARGIN])
        .with_position(at)
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top()
        .with_has_shadow(false)
}

impl eframe::App for Bench {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4] // fully transparent window background
    }

    #[allow(unused_variables)] // `frame` is only used on Windows
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let t = self.freeze.unwrap_or_else(|| self.start.elapsed().as_secs_f32());
        let ctx = ui.ctx().clone();
        let scale = ctx.pixels_per_point();
        if self.glows.as_ref().is_none_or(|g| g.scale != scale) {
            self.glows = Some(draw::Glows::new(&ctx, scale)); // glow images at this screen's scale
        }
        let glows = self.glows.as_ref().unwrap();
        let placing = self.total < PLACE_FRAMES;
        let whole = |ui: &mut egui::Ui, paint: &dyn Fn(&egui::Painter)| {
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| paint(ui.painter()));
        };
        if self.overlay {
            // grow to cover the primary monitor, then move to its corner (growing can shift it)
            if let (true, Some(size)) = (placing, ctx.input(|i| i.viewport().monitor_size)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(Pos2::ZERO));
                #[cfg(windows)]
                if !self.clipped {
                    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                    if let Ok(RawWindowHandle::Win32(h)) = frame.window_handle().map(|h| h.as_raw()) {
                        clip::to_rects(h.hwnd.get() as _, &clip::panels(self.windows, scale as f64));
                        self.clipped = true;
                    }
                }
            }
            whole(ui, &|p| (0..self.windows).for_each(|i| draw::panel(p, pos(i), t, glows)));
        } else {
            let origin = pos2(MARGIN, MARGIN);
            if placing {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos(0) - egui::vec2(MARGIN, MARGIN)));
            }
            whole(ui, &|p| draw::panel(p, origin, t, glows));
            for i in 1..self.windows {
                ctx.show_viewport_immediate(ViewportId::from_hash_of(i), viewport(i), |ui, _| {
                    if placing {
                        let at = pos(i) - egui::vec2(MARGIN, MARGIN);
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::OuterPosition(at));
                    }
                    whole(ui, &|p| draw::panel(p, origin, t, glows));
                });
            }
        }
        if self.total == 0 {
            println!("first-frame");
        }
        self.total += 1;
        self.frames += 1;
        if self.last.elapsed().as_secs_f32() >= 1.0 {
            println!("fps {:.0}", self.frames as f32 / self.last.elapsed().as_secs_f32());
            (self.frames, self.last) = (0, Instant::now());
        }
        // redraw every frame; vsync (eframe default) paces this to the display refresh rate
        ui.ctx().request_repaint();
    }
}

fn main() -> eframe::Result {
    env_logger::init(); // RUST_LOG=info shows backend choice and surface warnings
    // wgpu's default DX12 swapchain (from the HWND) and Vulkan are opaque on Windows (grey boxes);
    // only a swapchain on a DirectComposition visual keeps alpha. egui reads these at startup.
    #[cfg(windows)]
    for (key, value) in [("WGPU_BACKEND", "dx12"), ("WGPU_DX12_PRESENTATION_SYSTEM", "Visual")] {
        if std::env::var_os(key).is_none() {
            std::env::set_var(key, value);
        }
    }
    let windows = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
    let overlay = std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay");
    let mut viewport = viewport(0);
    if overlay {
        // shortcut: primary monitor only. On Windows the window is clipped to the panels on the first
        // frame; elsewhere every click passes through, widgets included.
        viewport = viewport.with_position([0.0, 0.0]).with_mouse_passthrough(!cfg!(windows));
    }
    // BENCH_RENDERER=glow picks OpenGL; with both features eframe would default to glow, so always set it
    let glow = std::env::var("BENCH_RENDERER").is_ok_and(|r| r == "glow");
    let renderer = if glow { eframe::Renderer::Glow } else { eframe::Renderer::Wgpu };
    let opts = eframe::NativeOptions { viewport, renderer, ..Default::default() };
    let now = Instant::now();
    let freeze = std::env::var("BENCH_FREEZE").ok().and_then(|t| t.parse().ok());
    let app = Bench {
        windows,
        start: now,
        frames: 0,
        total: 0,
        last: now,
        overlay,
        freeze,
        glows: None,
        #[cfg(windows)]
        clipped: false,
    };
    eframe::run_native(
        "egui 0",
        opts,
        Box::new(|cc| {
            // Space Mono for all monospace text, as in the reference
            let mut fonts = egui::FontDefinitions::default();
            let data = egui::FontData::from_static(halo::FONT);
            fonts.font_data.insert("space-mono".into(), std::sync::Arc::new(data));
            fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "space-mono".into());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(app))
        }),
    )
}
