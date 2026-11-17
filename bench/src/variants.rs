// The variants the bench runs, and on which platforms.
// (name, program, BENCH_MODE, extra env, platforms): "windows" = one window per widget, "overlay" =
// one click-through window over the primary monitor with every widget inside it. Program "qt" runs
// qt/main.py with a Python that has PySide6. Platforms are `env::consts::OS` values, empty = all.
pub type Variant = (&'static str, &'static str, &'static str, &'static [(&'static str, &'static str)], &'static [&'static str]);
const CANVAS: &[(&str, &str)] = &[("BENCH_DRAW", "canvas")];
const QT_GL: &[(&str, &str)] = &[("QSG_RHI_BACKEND", "opengl")];
const GLOW: &[(&str, &str)] = &[("BENCH_RENDERER", "glow")];
const FEMTOVG: &[(&str, &str)] = &[("SLINT_BACKEND", "winit-femtovg")];
const SKIA: &[(&str, &str)] = &[("SLINT_BACKEND", "winit-skia-opengl")];
const ALL: &[&str] = &[];
const WIN: &[&str] = &["windows"];
const LINUX: &[&str] = &["linux"];
const UNIX: &[&str] = &["linux", "macos"];
pub const VARIANTS: &[Variant] = &[
    ("idle (no app)", "", "", &[], ALL),
    ("tauri", "bench-tauri", "windows", &[], ALL),
    ("tauri overlay", "bench-tauri", "overlay", &[], ALL),
    ("tauri-canvas", "bench-tauri", "windows", CANVAS, ALL),
    ("tauri-canvas overlay", "bench-tauri", "overlay", CANVAS, ALL),
    ("egui-wgpu", "bench-egui", "windows", &[], ALL),
    ("egui-wgpu overlay", "bench-egui", "overlay", &[], ALL),
    ("egui-glow", "bench-egui", "windows", GLOW, LINUX),
    ("egui-glow overlay", "bench-egui", "overlay", GLOW, LINUX),
    ("qt", "qt", "windows", &[], ALL),
    ("qt overlay", "qt", "overlay", &[], ALL),
    ("qt-opengl", "qt", "windows", QT_GL, ALL),
    ("qt-opengl overlay", "qt", "overlay", QT_GL, ALL),
    ("slint-femtovg", "bench-slint", "windows", FEMTOVG, UNIX),
    ("slint-femtovg overlay", "bench-slint", "overlay", FEMTOVG, UNIX),
    ("slint-skia-gl", "bench-slint", "windows", SKIA, UNIX),
    ("slint-skia-gl overlay", "bench-slint", "overlay", SKIA, UNIX),
    ("gtk4-cairo", "bench-gtk", "windows", &[], LINUX),
    ("gtk4-cairo overlay", "bench-gtk", "overlay", &[], LINUX),
    ("win-d2d", "bench-win", "windows", &[], WIN),
    ("win-d2d overlay", "bench-win", "overlay", &[], WIN),
    ("win-cpp", "bench-win-cpp", "windows", &[], WIN),
    ("win-cpp overlay", "bench-win-cpp", "overlay", &[], WIN),
];

pub fn runs_here(os: &[&str]) -> bool {
    os.is_empty() || os.contains(&std::env::consts::OS)
}
