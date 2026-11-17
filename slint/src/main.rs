// Slint variant: BENCH_WINDOWS widgets, one transparent window each, or all in one click-through
// overlay window when BENCH_MODE=overlay. animation-tick() redraws every frame. Markup in ui.rs.
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, LogicalPosition, PhysicalPosition, RenderingState, Timer, TimerMode};
use std::{cell::Cell, rc::Rc, time::Duration};

mod ui;

use ui::{Glows, Overlay, Widget};

/// halo's glow images at the window's scale, handed to the panel markup.
fn set_glows(w: &slint::Window, glows: Glows) {
    let image = |s: &halo::Sprite| {
        let buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&s.rgba, s.width, s.height);
        slint::Image::from_rgba8_premultiplied(buf)
    };
    let scale = w.scale_factor();
    glows.set_outer(image(&halo::outer(scale)));
    glows.set_inner(image(&halo::inner(scale)));
    glows.set_title(image(&halo::title(scale)));
    let digits: Vec<slint::Image> = halo::digits(scale).iter().map(image).collect();
    glows.set_digits(slint::ModelRc::new(slint::VecModel::from(digits)));
}

static FIRST: std::sync::Once = std::sync::Once::new();

fn count_frames(w: &slint::Window) -> Timer {
    let frames = Rc::new(Cell::new(0u32));
    let f = frames.clone();
    w.set_rendering_notifier(move |state, _| {
        if let RenderingState::AfterRendering = state {
            FIRST.call_once(|| println!("first-frame"));
            f.set(f.get() + 1);
        }
    })
    .expect("rendering notifier");
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(1), move || println!("fps {}", frames.replace(0)));
    timer
}

fn main() -> Result<(), slint::PlatformError> {
    let n: usize = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
    let freeze: f32 = std::env::var("BENCH_FREEZE").ok().and_then(|t| t.parse().ok()).unwrap_or(-1.0);
    if std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay") {
        let o = Overlay::new()?;
        o.set_count(n as i32);
        o.set_freeze(freeze);
        let _timer = count_frames(o.window());
        o.window().set_position(PhysicalPosition::new(0, 0)); // before show: applied at creation
        o.show()?;
        // the native window only exists once the event loop runs, so size and place it from there
        let weak = o.as_weak();
        Timer::single_shot(Duration::from_millis(100), move || {
            let Some(o) = weak.upgrade() else { return };
            set_glows(o.window(), o.global::<Glows>());
            // shortcut: whole-window click-through and primary monitor only; real use needs per-widget hit regions
            let size = o.window().with_winit_window(|w| {
                w.set_cursor_hittest(false).ok();
                w.current_monitor().map(|m| m.size().to_logical::<f32>(m.scale_factor()))
            });
            if let Some(Some(size)) = size {
                o.set_w(size.width);
                o.set_h(size.height);
            }
            // growing can shift the window on macOS (it keeps the bottom-left corner): move it back
            let weak = o.as_weak();
            Timer::single_shot(Duration::from_millis(200), move || {
                if let Some(o) = weak.upgrade() {
                    o.window().set_position(PhysicalPosition::new(0, 0));
                }
            });
        });
        return slint::run_event_loop();
    }
    let mut keep = vec![];
    for i in 0..n {
        let w = Widget::new()?;
        w.set_freeze(freeze);
        // the window includes halo::MARGIN around the panel for its glow
        let at = LogicalPosition::new(40.0 + 260.0 * (i % 4) as f32 - halo::MARGIN, 40.0 + 140.0 * (i / 4) as f32 - halo::MARGIN);
        w.window().set_position(at);
        let timer = count_frames(w.window());
        w.show()?;
        // once the native window exists: glows at its scale, and the position again (macOS creates
        // borderless windows ~30 pt lower than asked)
        let weak = w.as_weak();
        Timer::single_shot(Duration::from_millis(100), move || {
            if let Some(w) = weak.upgrade() {
                set_glows(w.window(), w.global::<Glows>());
                w.window().set_position(at);
            }
        });
        keep.push((w, timer));
    }
    slint::run_event_loop()
}
