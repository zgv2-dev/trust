// Slint variant: BENCH_WINDOWS widgets, one transparent window each, or all in one click-through
// overlay window when BENCH_MODE=overlay. animation-tick() redraws every frame.
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, LogicalPosition, PhysicalPosition, RenderingState, Timer, TimerMode};
use std::{cell::Cell, rc::Rc, time::Duration};

slint::slint! {
    component Pie inherits Rectangle {
        in property <float> amount; // not `fill`: Path has its own fill brush
        width: 44px; height: 44px;
        border-width: 1px; border-radius: 22px; border-color: #50ff7880;
        Path {
            width: 44px; height: 44px;
            viewbox-width: 44; viewbox-height: 44;
            fill: #50ff7899;
            MoveTo { x: 22; y: 22; }
            LineTo { x: 22; y: 2; }
            ArcTo {
                radius-x: 20; radius-y: 20; large-arc: amount > 0.5; sweep: true;
                x: 22 + 20 * sin(amount * 360deg); y: 22 - 20 * cos(amount * 360deg);
            }
            Close {}
        }
    }

    component Panel inherits Rectangle {
        width: 240px; height: 110px;
        property <float> t: animation-tick() / 1s;
        background: #030a033c;
        border-width: 1px; border-radius: 4px; border-color: #50ff7859;
        VerticalLayout {
            padding: 10px; spacing: 4px;
            Text { text: "MOTORS"; color: #50ff78; font-family: "monospace"; }
            HorizontalLayout {
                spacing: 8px;
                for m in 4: VerticalLayout {
                    property <float> f: 0.5 + 0.45 * sin((t * 1.3 + m) * 1rad); // fake PWM
                    Pie { amount: f; }
                    Text { text: round(1000 + f * 1000); color: #50ff78; font-family: "monospace"; }
                }
            }
        }
    }

    export component Widget inherits Window {
        no-frame: true;
        always-on-top: true;
        background: transparent;
        width: 240px; height: 110px;
        Panel {}
    }

    export component Overlay inherits Window {
        in property <int> count;
        no-frame: true;
        always-on-top: true;
        background: transparent;
        for i in count: Panel {
            x: (40 + 260 * mod(i, 4)) * 1px;
            y: (40 + 140 * floor(i / 4)) * 1px;
        }
    }
}

fn count_frames(w: &slint::Window) -> Timer {
    let frames = Rc::new(Cell::new(0u32));
    let f = frames.clone();
    w.set_rendering_notifier(move |state, _| {
        if let RenderingState::AfterRendering = state {
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
    if std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay") {
        let o = Overlay::new()?;
        o.set_count(n as i32);
        let _timer = count_frames(o.window());
        o.show()?;
        // shortcut: whole-window click-through and primary monitor only; real use needs per-widget hit regions
        let size = o.window().with_winit_window(|w| {
            w.set_cursor_hittest(false).ok();
            w.current_monitor().map(|m| m.size())
        });
        if let Some(Some(size)) = size {
            o.window().set_position(PhysicalPosition::new(0, 0));
            o.window().set_size(slint::PhysicalSize::new(size.width, size.height));
        }
        return slint::run_event_loop();
    }
    let mut keep = vec![];
    for i in 0..n {
        let w = Widget::new()?;
        w.window().set_position(LogicalPosition::new(40.0 + 260.0 * (i % 4) as f32, 40.0 + 140.0 * (i / 4) as f32));
        let timer = count_frames(w.window());
        w.show()?;
        keep.push((w, timer));
    }
    slint::run_event_loop()
}
