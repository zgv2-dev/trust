# Transparent widget bench

Draws the same animated widget (pinged's Motors panel: 4 pies + numbers) with Tauri, egui and
Qt Quick, and measures what each costs. Each one redraws every frame, at the display refresh rate.

Every renderer runs in two modes:
- **windows**: one transparent, borderless, always-on-top window per widget (pinged today).
- **overlay**: one transparent window over the primary monitor with every widget drawn inside it; clicks between widgets pass through.

| variant | stack |
|---|---|
| tauri | Tauri 2 webview, HTML/CSS styled like pinged (grid, glow shadows), `requestAnimationFrame` |
| tauri-canvas | Same Tauri app drawing the reference itself (`tauri/dist/widget.js`) on one `<canvas>`; static parts drawn once and reused |
| egui-wgpu | eframe + wgpu (Metal / DX12 / Vulkan). On Windows forced to DX12 on a DirectComposition visual (`WGPU_DX12_PRESENTATION_SYSTEM=Visual`); the default HWND swapchain is opaque |
| egui-glow | Linux only. Same egui app on OpenGL (`BENCH_RENDERER=glow`) |
| qt | Qt Quick (QML) through PySide6, default renderer (Metal / D3D11 / OpenGL), MultiEffect glow |
| qt-opengl | Same, forced to OpenGL (`QSG_RHI_BACKEND=opengl`), in case the default isn't transparent |
| slint-femtovg | Linux and macOS. Slint, femtovg OpenGL renderer |
| slint-skia-gl | Linux and macOS. Slint, Skia OpenGL renderer (better text than femtovg) |
| gtk4-cairo | Linux only. GTK 4 window, widget drawn with Cairo (CPU), composited by GTK's GL/Vulkan renderer |
| win-d2d | Windows only. Native Win32 + DirectComposition + Direct2D/DirectWrite, GPU shadow for the glow, one `DwmFlush` per frame |
| win-cpp | Windows only. Same as win-d2d, written in C++ (`win-cpp/src/*.cpp`), to rule out Rust/`windows` crate overhead |

## Run

You need Rust stable on every platform. On Windows, win-cpp also needs the MSVC C++ build tools
(Visual Studio "Desktop development with C++", which Rust's MSVC toolchain already requires). 
On Linux (Debian/Ubuntu package names; compile-checked on Debian 13):

```
sudo apt install build-essential curl libwebkit2gtk-4.1-dev libxdo-dev libssl-dev librsvg2-dev \
  libayatana-appindicator3-dev libgtk-4-dev libfontconfig-dev libxkbcommon-dev clang
```

The first line is what Tauri needs, the same as pinged. `libgtk-4-dev` is for gtk4-cairo and the
last two for Slint. The bench builds each variant on its own, so a missing package skips only
that variant's rows (`skipped: build failed`), with the compiler error above the results.

The Qt rows need Python with PySide6: `pip install PySide6`. The bench uses the first of
`$BENCH_PYTHON`, `python3`, `python` that can import it, and marks the Qt rows skipped if none can.
With a virtualenv, set `BENCH_PYTHON` to its interpreter.

### The reference look

`reference/index.html` is the look every renderer is scored against: the widgets drawn on a
canvas (`tauri/dist/widget.js`) over a random background. Open it in a browser at 100% zoom on
the machine you'll test, check the look over a few backgrounds (Space for the next one), then
press **Export reference PNG** and move the file (`ref-<os>-<scale>x.png`) into `reference/`.
Export again after any change to `widget.js`, and on each OS: text rendering differs between
systems. The font is Space Mono (`fonts/`, SIL OFL), embedded as `tauri/dist/space-mono.js` because
browsers won't load font files from `file://`. After changing the font, regenerate that file:

```
{ head -2 tauri/dist/space-mono.js; printf 'window.SPACE_MONO = "data:font/ttf;base64,'; \
  base64 -i fonts/SpaceMono-Regular.ttf | tr -d '\n'; echo '";'; } > space-mono.tmp && mv space-mono.tmp tauri/dist/space-mono.js
```

The PNG includes 8 px around the panel for its outer glow. With separate 240x110 windows the
glow falls outside the window, and the look error counts that.

### The bench

The look check takes screenshots. On macOS, give your terminal Screen Recording permission
(System Settings → Privacy & Security); without it the screenshots show only the wallpaper and
every widget reads as missing. On Linux it uses the first of `grim`, `spectacle`,
`gnome-screenshot`, `import` (ImageMagick) that works.

```
cargo run --release -p bench
```

This builds every variant, then runs them one at a time. For each one it takes a screenshot,
launches the app with the animation frozen (cold start), waits 3 s, takes another screenshot and
closes it, launches it 3 more times (warm starts), then measures 5 s warmup + 20 s with the
animation running. About 45 s per variant, 10–15 minutes in total. Windows open and close on
their own. Don't touch the machine or move windows while it runs, and keep other apps idle.
For the whole run a backdrop window (`backdrop/`, a fixed checker pattern) covers the top-left
1140 × 340 logical px, so the widgets are always photographed over the same background. The
widgets are always-on-top windows and draw over it. On Wayland the compositor places the backdrop
wherever it likes, so it may not sit behind the widgets. A third screenshot after each app closes
also marks pixels that changed by themselves, and the look error skips those.

Each run writes `results/<os>-<arch>-<UTC time>.md` and a folder of the same name with one
picture per variant: the expected look on top, the actual screen below. Runs accumulate.
`cargo run --release -p bench -- --summary` writes `results/<os>-<arch>-summary.md` with the
median of every column across all runs, so one noisy run doesn't decide anything. Send the result
files (and pictures of variants that look wrong) back.

Options: `--windows N` (the widget count, default 4), `--secs N`, `--warmup N`,
`--only "<variant name>"` (prints only, doesn't save), `--no-build`, `--scale S` (which reference
to use when `reference/` has more than one for this OS), `--purge` (see cold start below), and
`--summary`.

Please also note the display refresh rate and resolution, and on Linux, Wayland or X11 plus the desktop (GNOME, KDE, …) and the GPU driver.
Also say which variants drew something wrong: an opaque or black background, a missing widget, or
widgets that open in odd places (on Linux some variants can't place windows, see below).

## Columns

- **app CPU %**: the app's whole process tree, as a share of one core (100 = one full core). On
  macOS it also counts the WebKit helper processes that launchd starts for the webview.
- **app RSS MB**: peak memory across those processes.
- **compositor CPU %**: WindowServer / dwm.exe / gnome-shell / kwin / Xorg. Transparent windows
  also cost the compositor, and an overlay can move cost there. Compare it with the idle row.
  Read it as a rough signal: other apps on the desktop affect it too.
- **fps per window**: frames actually drawn per second. It should match the refresh rate. Below
  that, the variant can't keep up. Above it, vsync isn't working and the CPU figure is inflated.
- **cold start ms**: launch to first frame drawn, on the variant's first launch in the run. A first
  launch still finds most files in the OS cache. For a real cold start, run `sudo -v`, then the
  bench with `--purge`: it drops the file cache before each cold launch (macOS `purge`, Linux
  `drop_caches`; Windows has no equivalent).
- **warm start ms**: the same, median of 3 launches right after.
- **widgets found**: how many of the widgets appear on screen in the screenshot.
- **placement off px**: the farthest any found widget sits from where it should be, in logical
  pixels. Windows the window manager moved, or an overlay pushed below the menu bar, show here.
- **look error %**: how far the widgets look from the reference: per-pixel difference from the
  reference composited over the same background, relative to drawing nothing there. 0 = identical,
  100 = as different as an empty spot. Missing widgets count as 100. Anti-aliasing and font
  rendering alone give a few percent; compare variants with each other.

## Known gaps

- GPU time isn't measured, only CPU.
- "First frame" is when the app finished its first draw call (Tauri: the first `requestAnimationFrame`),
  not when the compositor showed it.
- Every overlay is clipped to its panels: clicks between widgets fall through to the app below, and the
  widgets stay clickable. Qt uses `QWindow::setMask` (all platforms); tauri, egui and win-d2d use
  `SetWindowRgn` (`clip` crate) on Windows, and off Windows tauri and egui pass every click through.
- Every overlay covers only the primary monitor. A real one needs one window per monitor.
- softbuffer (CPU raster) is left out: version 0.4 ignores alpha on macOS, so its windows would be opaque.
- egui-glow runs only on Linux: no vsync on macOS, and on Windows the first window was black and the overlay drew nothing.
- Slint is skipped on Windows: 25–38 fps with separate windows, and poor text. Its numbers are in `results/archive.md`.
  Its overlay passes every click through, widgets included.
- On Wayland no app can place its own windows or keep them on top, so with separate windows the
  compositor picks where they go. GTK 4 can't do either on X11 too, and its overlay is a fullscreen window.
  A real always-on-top overlay on Wayland needs the layer-shell protocol (KDE, Sway, Hyprland; not GNOME).
- gtk4-cairo has no glow.
- win-cpp's `main` is a 3-line Rust shim that calls the C++ `bench_main`, so cargo builds it; nothing else is Rust.
- The Qt process is Python, but Python only loads the QML; animation and drawing run in Qt's C++.
- The compositor process list is a fixed set of names. A compositor missing from it reads 0.
