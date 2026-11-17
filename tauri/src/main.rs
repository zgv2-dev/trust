// Tauri variant: BENCH_WINDOWS widgets, one transparent webview window each, or all in one
// click-through overlay window when BENCH_MODE=overlay. Animates with requestAnimationFrame;
// BENCH_DRAW=canvas draws on a <canvas> instead of styled DOM.
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn fps(n: f64) {
    println!("fps {n:.0}");
}

#[tauri::command]
fn first_frame() {
    println!("first-frame");
}

fn main() {
    let n: usize = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
    let page = if std::env::var("BENCH_DRAW").is_ok_and(|d| d == "canvas") { "canvas.html" } else { "index.html" };
    // BENCH_FREEZE: draw that instant forever, so screenshots can be compared with the reference
    let freeze = std::env::var("BENCH_FREEZE").ok().and_then(|t| t.parse::<f64>().ok());
    let freeze = freeze.map_or("undefined".into(), |t| t.to_string());
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fps, first_frame])
        .setup(move |app| {
            let freeze = &freeze;
            if std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay") {
                // shortcut: primary monitor only; real use needs one overlay per monitor
                let m = app.primary_monitor()?.expect("a monitor");
                let size = m.size().to_logical::<f64>(m.scale_factor());
                let w = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App(page.into()))
                    .initialization_script(format!("window.BENCH_N = {n}; window.BENCH_FREEZE = {freeze};"))
                    .title("tauri overlay")
                    .inner_size(size.width, size.height)
                    .position(0.0, 0.0)
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(true)
                    .shadow(false)
                    .build()?;
                #[cfg(windows)]
                clip::to_rects(w.hwnd()?.0, &clip::panels(n, m.scale_factor()));
                // shortcut: off Windows every click passes through, widgets included; add a hit mask there when needed
                #[cfg(not(windows))]
                w.set_ignore_cursor_events(true)?;
                return Ok(());
            }
            for i in 0..n {
                WebviewWindowBuilder::new(app, format!("w{i}"), WebviewUrl::App(page.into()))
                    .initialization_script(format!("window.BENCH_FREEZE = {freeze};"))
                    .title(format!("tauri {i}"))
                    // the panel plus 8 px on every side for its outer glow (MARGIN in widget.js)
                    .inner_size(240.0 + 16.0, 110.0 + 16.0)
                    .position(40.0 + 260.0 * (i % 4) as f64 - 8.0, 40.0 + 140.0 * (i / 4) as f64 - 8.0)
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(true)
                    .shadow(false)
                    .build()?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri app");
}

