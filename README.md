# Transparent widget bench

Draws the same animated widget (pinged's Motors panel: 4 pies + numbers) with Tauri, egui and
Qt Quick, and measures what each costs. Each one redraws every frame, at the display refresh rate.

Every renderer runs in two modes:
- **windows**: one transparent, borderless, always-on-top window per widget (pinged today).
- **overlay**: one transparent window over the primary monitor with every widget drawn inside it; clicks between widgets pass through.

| variant | stack |
|---|---|
| tauri | Tauri 2 webview, HTML/CSS styled like pinged (grid, glow shadows), `requestAnimationFrame` |
| tauri-canvas | Same Tauri app, widget drawn on one `<canvas>`; static parts and glow drawn once and reused |
| egui-wgpu | eframe + wgpu (Metal / DX12 / Vulkan). On Windows forced to DX12 on a DirectComposition visual (`WGPU_DX12_PRESENTATION_SYSTEM=Visual`); the default HWND swapchain is opaque |
| qt | Qt Quick (QML) through PySide6, default renderer (Metal / D3D11 / OpenGL), MultiEffect glow |
| qt-opengl | Same, forced to OpenGL (`QSG_RHI_BACKEND=opengl`), in case the default isn't transparent |
| win-d2d | Windows only. Native Win32 + DirectComposition + Direct2D/DirectWrite, GPU shadow for the glow, one `DwmFlush` per frame |
| win-cpp | Windows only. Same as win-d2d, written in C++ (`win-cpp/src/*.cpp`), to rule out Rust/`windows` crate overhead |

## Run

You need Rust stable on every platform. On Windows, win-cpp also needs the MSVC C++ build tools
(Visual Studio "Desktop development with C++", which Rust's MSVC toolchain already requires). On Linux you also need the Tauri system packages
(`libwebkit2gtk-4.1-dev` and its companions), the same ones pinged builds with.

The Qt rows need Python with PySide6: `pip install PySide6`. The bench uses the first of
`$BENCH_PYTHON`, `python3`, `python` that can import it, and marks the Qt rows skipped if none can.
With a virtualenv, set `BENCH_PYTHON` to its interpreter.

```
cargo run --release -p bench
```

This builds every variant, then runs them one at a time: 5 s warmup and 20 s measured each,
about 5–6 minutes in total. Windows open and close on their own. Don't touch the machine or
move windows while it runs, and keep other apps idle. Results print at the end and are saved to
`results-<os>-<arch>.md`. Send that file back.

Options: `--windows N` (the widget count, default 4), `--secs N`, `--warmup N`,
`--only "<variant name>"` (prints only, doesn't save), and `--no-build`.

Please also note the display refresh rate and resolution, and on Linux, Wayland or X11 plus the desktop (GNOME, KDE, …).

## Columns

- **app CPU %**: the app's whole process tree, as a share of one core (100 = one full core). On
  macOS it also counts the WebKit helper processes that launchd starts for the webview.
- **app RSS MB**: peak memory across those processes.
- **compositor CPU %**: WindowServer / dwm.exe / gnome-shell / kwin / Xorg. Transparent windows
  also cost the compositor, and an overlay can move cost there. Compare it with the idle row.
  Read it as a rough signal: other apps on the desktop affect it too.
- **fps per window**: frames actually drawn per second. It should match the refresh rate. Below
  that, the variant can't keep up. Above it, vsync isn't working and the CPU figure is inflated.

## Known gaps

- GPU time isn't measured, only CPU.
- Every overlay is clipped to its panels: clicks between widgets fall through to the app below, and the
  widgets stay clickable. Qt uses `QWindow::setMask` (all platforms); tauri, egui and win-d2d use
  `SetWindowRgn` (`clip` crate) on Windows, and off Windows tauri and egui pass every click through.
- Every overlay covers only the primary monitor. A real one needs one window per monitor.
- softbuffer (CPU raster) is left out: version 0.4 ignores alpha on macOS, so its windows would be opaque.
- egui-glow is left out: no vsync on macOS, and on Windows the first window was black and the overlay drew nothing.
- Slint is left out: 25–38 fps with separate windows on Windows, and poor text. Its numbers are in `results/archive.md`.
- win-cpp's `main` is a 3-line Rust shim that calls the C++ `bench_main`, so cargo builds it; nothing else is Rust.
- The Qt process is Python, but Python only loads the QML; animation and drawing run in Qt's C++.
- The compositor process list is a fixed set of names. A compositor missing from it reads 0.
