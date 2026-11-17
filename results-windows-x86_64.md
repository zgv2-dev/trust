# Widget bench: Windows 11 Pro (x86_64), Intel(R) Core(TM) 5 210H, 12 logical CPUs

4 widgets, 5s warmup, 20s measured. CPU % is of one core (100 = one full core).

| variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |
|---|---|---|---|---|---|---|
| idle (no app) | 0.0 | 0.0 | 0 | 0 | 18.2 | - |
| tauri | 175.1 | 227.7 | 867 | 10 | 34.3 | 127 |
| tauri overlay | 121.1 | 168.1 | 446 | 7 | 29.8 | 137 |
| tauri-canvas | 134.0 | 196.9 | 677 | 10 | 37.5 | 132 |
| tauri-canvas overlay | 81.2 | 110.4 | 416 | 7 | 31.9 | 142 |
| egui-wgpu | 53.5 | 132.8 | 349 | 1 | 30.2 | 131 |
| egui-wgpu overlay | 19.9 | 26.3 | 221 | 1 | 21.8 | 144 |
| qt | 82.2 | 127.9 | 194 | 2 | 18.8 | 116 |
| qt overlay | 29.9 | 50.8 | 143 | 2 | 12.6 | 144 |
| qt-opengl | 140.9 | 183.1 | 390 | 2 | 16.5 | 76 |
| qt-opengl overlay | 43.7 | 55.5 | 201 | 2 | 1.2 | 144 |
| win-d2d | 66.1 | 104.5 | 73 | 1 | 17.2 | 116 |
| win-d2d overlay | 34.5 | 45.3 | 85 | 1 | 3.7 | 143 |
| win-cpp | 66.5 | 92.4 | 73 | 1 | 11.1 | 125 |
| win-cpp overlay | 33.3 | 40.0 | 85 | 1 | 5.0 | 143 |
