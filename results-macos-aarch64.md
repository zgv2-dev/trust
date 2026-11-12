# Widget bench: macOS 27.3 (aarch64), Apple M3, 8 logical CPUs

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
