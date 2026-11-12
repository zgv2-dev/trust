// egui variant: BENCH_WINDOWS widgets, one transparent window each, or all in one click-through
// overlay window when BENCH_MODE=overlay. Renders with wgpu.
use eframe::egui::{self, Color32, Pos2, Shape, Stroke, Vec2, ViewportBuilder, ViewportId};
use std::time::Instant;

const ACCENT: Color32 = Color32::from_rgb(80, 255, 120);

struct Bench {
    windows: usize,
    start: Instant,
    frames: u32,
    last: Instant,
    overlay: bool,
    sized: bool,
}

fn pos(i: usize) -> [f32; 2] {
    [40.0 + 260.0 * (i % 4) as f32, 40.0 + 140.0 * (i / 4) as f32]
}

fn viewport(i: usize) -> ViewportBuilder {
    ViewportBuilder::default()
        .with_title(format!("egui {i}"))
        .with_inner_size([240.0, 110.0])
        .with_position(pos(i))
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top()
        .with_has_shadow(false)
}

impl eframe::App for Bench {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4] // fully transparent window background
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let t = self.start.elapsed().as_secs_f32();
        let ctx = ui.ctx().clone();
        if self.overlay {
            // grow to cover the primary monitor once its size is known
            if let (false, Some(size)) = (self.sized, ctx.input(|i| i.viewport().monitor_size)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(Pos2::ZERO));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                self.sized = true;
            }
            for i in 0..self.windows {
                egui::Area::new(egui::Id::NULL.with(("widget", i))).fixed_pos(pos(i)).show(&ctx, |ui| panel(ui, t));
            }
        } else {
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| panel(ui, t));
            for i in 1..self.windows {
                ctx.show_viewport_immediate(ViewportId::from_hash_of(i), viewport(i), |ui, _| {
                    egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| panel(ui, t));
                });
            }
        }
        self.frames += 1;
        if self.last.elapsed().as_secs_f32() >= 1.0 {
            println!("fps {:.0}", self.frames as f32 / self.last.elapsed().as_secs_f32());
            (self.frames, self.last) = (0, Instant::now());
        }
        // redraw every frame; vsync (eframe default) paces this to the display refresh rate
        ui.ctx().request_repaint();
    }
}

fn panel(ui: &mut egui::Ui, t: f32) {
    let panel = egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(3, 10, 3, 60))
        .stroke(Stroke::new(1.0, ACCENT.gamma_multiply(0.35)))
        .corner_radius(4.0)
        .inner_margin(10.0);
    panel.show(ui, |ui| {
        ui.set_min_size(Vec2::new(220.0, 90.0));
        ui.label(egui::RichText::new("MOTORS").monospace().color(ACCENT));
        ui.horizontal(|ui| {
            for m in 0..4 {
                let fill = 0.5 + 0.45 * (t * 1.3 + m as f32).sin(); // fake PWM
                ui.vertical(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(44.0), egui::Sense::hover());
                    pie(ui.painter(), rect.center(), 20.0, fill);
                    let text = format!("{:4.0}", 1000.0 + fill * 1000.0);
                    ui.label(egui::RichText::new(text).monospace().color(ACCENT));
                });
            }
        });
    });
}

fn pie(p: &egui::Painter, c: Pos2, r: f32, fill: f32) {
    p.circle_stroke(c, r, Stroke::new(1.0, ACCENT.gamma_multiply(0.5)));
    let steps = 32;
    let mut pts = vec![c];
    for i in 0..=steps {
        let a = -std::f32::consts::FRAC_PI_2 + fill * std::f32::consts::TAU * i as f32 / steps as f32;
        pts.push(c + r * Vec2::angled(a));
    }
    p.add(Shape::convex_polygon(pts, ACCENT.gamma_multiply(0.6), Stroke::NONE));
}

fn main() -> eframe::Result {
    let windows = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
    // wgpu's default DX12 swapchain (DxgiFromHwnd) ignores alpha; the DirectComposition one keeps it
    if cfg!(windows) && std::env::var_os("WGPU_DX12_PRESENTATION_SYSTEM").is_none() {
        std::env::set_var("WGPU_DX12_PRESENTATION_SYSTEM", "Visual");
    }
    let overlay = std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay");
    let mut viewport = viewport(0);
    if overlay {
        // shortcut: whole-window click-through and primary monitor only; real use needs per-widget hit regions
        viewport = viewport.with_position([0.0, 0.0]).with_mouse_passthrough(true);
    }
    let opts = eframe::NativeOptions { viewport, ..Default::default() };
    let now = Instant::now();
    let app = Bench { windows, start: now, frames: 0, last: now, overlay, sized: false };
    eframe::run_native("egui 0", opts, Box::new(|_| Ok(Box::new(app))))
}
