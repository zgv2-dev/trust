// GTK 4 variant (Linux): Cairo drawing in a DrawingArea, composited by GTK's GPU renderer (GL or
// Vulkan). BENCH_WINDOWS widgets, one window each, or all in one fullscreen overlay window when
// BENCH_MODE=overlay, where clicks outside the panels pass through.
#[cfg(target_os = "linux")]
mod draw;

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("bench-gtk only runs on Linux");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() -> gtk::glib::ExitCode {
    use gtk::prelude::*;
    let n: usize = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
    let overlay = std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay");
    let app = gtk::Application::builder()
        .application_id("dev.bench.gtk")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| app::activate(app, n, overlay));
    app.run_with_args::<&str>(&[])
}

#[cfg(target_os = "linux")]
mod app {
    use crate::draw::{self, H, W};
    use gtk::{cairo, gdk, glib, prelude::*};
    use std::{cell::Cell, rc::Rc, time::Duration, time::Instant};

    fn origin(i: usize) -> (f64, f64) {
        (40.0 + 260.0 * (i % 4) as f64, 40.0 + 140.0 * (i / 4) as f64)
    }

    fn panels(n: usize) -> cairo::Region {
        let rect = |(x, y): (f64, f64)| cairo::RectangleInt::new(x as i32, y as i32, W as i32, H as i32);
        cairo::Region::create_rectangles(&(0..n).map(|i| rect(origin(i))).collect::<Vec<_>>())
    }

    pub fn activate(app: &gtk::Application, n: usize, overlay: bool) {
        let css = gtk::CssProvider::new();
        css.load_from_data("window { background: transparent; }");
        let display = gdk::Display::default().expect("no display");
        gtk::style_context_add_provider_for_display(&display, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        let layouts: Vec<Vec<(f64, f64)>> =
            if overlay { vec![(0..n).map(origin).collect()] } else { (0..n).map(|_| vec![(0.0, 0.0)]).collect() };
        let windows = layouts.len() as u32;
        let (start, frames) = (Instant::now(), Rc::new(Cell::new(0u32)));
        for origins in layouts {
            let window = gtk::Window::builder().application(app).decorated(false).resizable(false).build();
            let area = gtk::DrawingArea::new();
            let f = frames.clone();
            area.set_draw_func(move |_, cr, _, _| {
                f.set(f.get() + 1);
                let t = start.elapsed().as_secs_f64();
                for &(x, y) in &origins {
                    let _ = draw::panel(cr, x, y, t);
                }
            });
            // redraw on every frame-clock tick: vsync-paced, like requestAnimationFrame
            area.add_tick_callback(|area, _| {
                area.queue_draw();
                glib::ControlFlow::Continue
            });
            window.set_child(Some(&area));
            if overlay {
                // shortcut: GTK 4 can't place windows, so the overlay is fullscreen on the monitor the WM picks
                window.fullscreen();
                window.connect_map(move |w| w.surface().unwrap().set_input_region(Some(&panels(n))));
            } else {
                // shortcut: GTK 4 has no API for window position or always-on-top; the WM places them
                window.set_default_size(W as i32, H as i32);
            }
            window.present();
        }
        glib::timeout_add_local(Duration::from_secs(1), move || {
            println!("fps {}", frames.replace(0) / windows);
            glib::ControlFlow::Continue
        });
    }
}
