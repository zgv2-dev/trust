# Archived results

Runs from before slint and egui-glow were dropped. Kept for the record.

## macOS, 2026-11-12

Valid: every variant was transparent. egui-glow ignored vsync (fps above 60).

4 widgets, 5s warmup, 20s measured. CPU % is of one core (100 = one full core).

| variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |
|---|---|---|---|---|---|---|
| idle (no app) | 0.0 | 0.0 | 0 | 0 | 1.4 | - |
| tauri | 65.3 | 69.3 | 360 | 7 | 37.6 | 60 |
| tauri overlay | 53.7 | 57.3 | 188 | 4 | 38.7 | 60 |
| egui-wgpu | 34.0 | 39.9 | 106 | 1 | 38.6 | 60 |
| egui-wgpu overlay | 17.5 | 18.5 | 101 | 1 | 41.0 | 60 |
| egui-glow | 18.3 | 19.3 | 107 | 1 | 20.3 | 74 |
| egui-glow overlay | 11.3 | 12.4 | 105 | 1 | 29.2 | 90 |
| slint-femtovg | 15.8 | 16.4 | 99 | 1 | 24.5 | 60 |
| slint-femtovg overlay | 7.9 | 10.2 | 88 | 1 | 23.3 | 60 |

## Windows 11, 2026-11-12, run 1

Normal user, so dwm.exe CPU couldn't be read (reads 0). egui-wgpu windows were opaque grey, so its
numbers are invalid. egui-glow's first window was black and its overlay drew nothing.

Intel Core 5 210H, 12 logical CPUs, 144 Hz display. 4 widgets, 5s warmup, 20s measured.

| variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |
|---|---|---|---|---|---|---|
| idle (no app) | 0.0 | 0.0 | 0 | 0 | 0.0 | - |
| tauri | 75.6 | 133.8 | 935 | 10 | 0.0 | 144 |
| tauri overlay | 81.2 | 96.8 | 427 | 7 | 0.0 | 144 |
| egui-wgpu | 11.5 | 45.5 | 393 | 1 | 0.0 | 144 |
| egui-wgpu overlay | 26.1 | 37.1 | 224 | 1 | 0.0 | 144 |
| egui-glow | 45.6 | 52.3 | 191 | 1 | 0.0 | 115 |
| egui-glow overlay | 23.6 | 37.5 | 105 | 1 | 0.0 | 144 |
| slint-femtovg | 35.7 | 61.6 | 204 | 1 | 0.0 | 38 |
| slint-femtovg overlay | 15.6 | 35.7 | 105 | 1 | 0.0 | 115 |

## Windows 11, 2026-11-12, run 2

Administrator, but screen recording was running, which loaded dwm and lowered fps. Transparency
was the same as run 1. Slint text quality was poor.

4 widgets, 5s warmup, 20s measured. CPU % is of one core (100 = one full core).

| variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |
|---|---|---|---|---|---|---|
| idle (no app) | 0.0 | 0.0 | 0 | 0 | 51.8 | - |
| tauri | 170.3 | 225.4 | 900 | 10 | 91.2 | 144 |
| tauri overlay | 88.5 | 109.1 | 429 | 7 | 88.0 | 144 |
| egui-wgpu | 53.0 | 68.2 | 348 | 1 | 91.0 | 107 |
| egui-wgpu overlay | 20.0 | 35.4 | 222 | 1 | 88.2 | 79 |
| egui-glow | 33.0 | 41.4 | 130 | 1 | 87.2 | 75 |
| egui-glow overlay | 27.1 | 35.4 | 100 | 1 | 74.5 | 122 |
| slint-femtovg | 28.4 | 41.5 | 133 | 1 | 88.7 | 25 |
| slint-femtovg overlay | 18.6 | 33.4 | 104 | 1 | 97.1 | 81 |
