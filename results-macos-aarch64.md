# Widget bench: macOS 27.3 (aarch64), Apple M3, 8 logical CPUs

4 widgets, 5s warmup, 20s measured. CPU % is of one core (100 = one full core).

| variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |
|---|---|---|---|---|---|---|
| idle (no app) | 0.0 | 0.0 | 0 | 0 | 40.5 | - |
| tauri | 71.1 | 74.0 | 360 | 7 | 42.2 | 60 |
| tauri overlay | 53.1 | 54.8 | 187 | 4 | 42.3 | 60 |
| tauri-canvas | 47.0 | 51.1 | 320 | 7 | 42.0 | 60 |
| tauri-canvas overlay | 27.2 | 28.5 | 185 | 4 | 41.9 | 60 |
| egui-wgpu | 29.8 | 32.3 | 106 | 1 | 41.6 | 60 |
| egui-wgpu overlay | 13.1 | 13.8 | 102 | 1 | 41.9 | 60 |
| qt | 36.9 | 38.4 | 141 | 1 | 40.5 | 58 |
| qt overlay | 22.9 | 24.0 | 133 | 1 | 41.3 | 60 |
| qt-opengl | 27.3 | 29.0 | 150 | 1 | 39.7 | 60 |
| qt-opengl overlay | 22.7 | 23.9 | 138 | 1 | 41.6 | 60 |
