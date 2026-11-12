// Tauri variant: BENCH_WINDOWS widgets, one transparent webview window each, or all in one
// click-through overlay window when BENCH_MODE=overlay. Animates with requestAnimationFrame.
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn fps(n: f64) {
    println!("fps {n:.0}");
}

fn main() {
    let n: usize = std::env::var("BENCH_WINDOWS").ok().and_then(|n| n.parse().ok()).unwrap_or(4);
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fps])
        .setup(move |app| {
            if std::env::var("BENCH_MODE").is_ok_and(|m| m == "overlay") {
                // shortcut: whole-window click-through and primary monitor only; real use needs per-widget hit regions
                let m = app.primary_monitor()?.expect("a monitor");
                let size = m.size().to_logical::<f64>(m.scale_factor());
                let w = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html".into()))
                    .initialization_script(format!("window.BENCH_N = {n};"))
                    .title("tauri overlay")
                    .inner_size(size.width, size.height)
                    .position(0.0, 0.0)
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(true)
                    .shadow(false)
                    .build()?;
                w.set_ignore_cursor_events(true)?;
                return Ok(());
            }
            for i in 0..n {
                WebviewWindowBuilder::new(app, format!("w{i}"), WebviewUrl::App("index.html".into()))
                    .title(format!("tauri {i}"))
                    .inner_size(240.0, 110.0)
                    .position(40.0 + 260.0 * (i % 4) as f64, 40.0 + 140.0 * (i / 4) as f64)
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
