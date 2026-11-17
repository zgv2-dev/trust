// Static backdrop behind the widgets while the bench runs: one borderless, opaque window over the
// strip the widgets use, painted once with a fixed pattern. Screenshots then compare the widgets
// against a background that never changes, instead of whatever terminal or window sits there.
use eframe::egui::{self, Color32, Pos2, Rect, Vec2, ViewportBuilder};

/// Covers the widget grid (panels at 40 + 260 i, 60 + 140 j, 240 x 110, plus glow), logical px.
const SIZE: [f32; 2] = [1140.0, 340.0];
const CELL: f32 = 16.0;

struct Backdrop;

impl eframe::App for Backdrop {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        // checkerboard over a diagonal blue-to-orange gradient: busy enough that an opaque or
        // misplaced widget stands out, and identical every run (no randomness, no animation)
        let p = ui.painter();
        let (cols, rows) = ((SIZE[0] / CELL).ceil() as usize, (SIZE[1] / CELL).ceil() as usize);
        for r in 0..rows {
            for c in 0..cols {
                let f = (c + r) as f32 / (cols + rows) as f32;
                let lift = if (c + r) % 2 == 0 { 0 } else { 28 };
                let mix = |a: f32, b: f32| (a + (b - a) * f) as u8 + lift;
                let color = Color32::from_rgb(mix(30.0, 200.0), mix(60.0, 110.0), mix(150.0, 40.0));
                let min = Pos2::new(c as f32 * CELL, r as f32 * CELL);
                p.rect_filled(Rect::from_min_size(min, Vec2::splat(CELL)), 0.0, color);
            }
        }
    }
}

fn main() -> eframe::Result {
    let viewport = ViewportBuilder::default()
        .with_title("bench backdrop")
        .with_position([0.0, 0.0])
        .with_inner_size(SIZE)
        .with_decorations(false)
        .with_resizable(false);
    let opts = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("bench backdrop", opts, Box::new(|_| Ok(Box::new(Backdrop))))
}
