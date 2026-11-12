# Transparent widget bench

Draws the same animated widget (pinged's Motors panel: 4 pies + numbers) with Tauri and pure-Rust
renderers, and measures what each costs. Each one redraws every frame, at the display refresh rate.

Every renderer runs in two modes:
- **windows**: one transparent, borderless, always-on-top window per widget (pinged today).
- **overlay**: one click-through transparent window over the primary monitor, with every widget drawn inside it.

| variant | stack |
|---|---|
| tauri | Tauri 2 webview, HTML/CSS styled like pinged (grid, glow shadows), `requestAnimationFrame` |
| egui-wgpu | eframe + wgpu (Metal / DX12 / Vulkan) |
| egui-glow | eframe + OpenGL |
| slint-femtovg | Slint, femtovg OpenGL renderer |

## Run

You need Rust stable on every platform. On Linux you also need the Tauri system packages
(`libwebkit2gtk-4.1-dev` and its companions), the same ones pinged builds with.

```
cargo run --release -p bench
```

This builds every variant, then runs them one at a time: 5 s warmup and 20 s measured each,
about 4–5 minutes in total. Windows open and close on their own. Don't touch the machine or
move windows while it runs, and keep other apps idle. Results print at the end and are saved to
`results-<os>-<arch>.md`. Send that file back.

Options: `--windows N` (the widget count, default 4), `--secs N`, `--warmup N`,
`--only "<variant name>"`, and `--no-build`.

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
- The overlay makes its whole window click-through and covers only the primary monitor. A real
  overlay needs per-widget hit regions and one window per monitor.
- softbuffer (CPU raster) is left out: version 0.4 ignores alpha on macOS, so its windows would be opaque.
- The compositor process list is a fixed set of names. A compositor missing from it reads 0.
