// Runs each widget variant with the same scene and window count, and reports what it costs.
mod sample;

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, fs, thread};
use sysinfo::{Pid, System};

// (name, binary, args, BENCH_MODE): "windows" = one window per widget, "overlay" = one click-through
// window over the primary monitor with every widget inside it.
const VARIANTS: &[(&str, &str, &[&str], &str)] = &[
    ("idle (no app)", "", &[], ""),
    ("tauri", "bench-tauri", &[], "windows"),
    ("tauri overlay", "bench-tauri", &[], "overlay"),
    ("egui-wgpu", "bench-egui", &["wgpu"], "windows"),
    ("egui-wgpu overlay", "bench-egui", &["wgpu"], "overlay"),
    ("egui-glow", "bench-egui", &["glow"], "windows"),
    ("egui-glow overlay", "bench-egui", &["glow"], "overlay"),
    ("slint-femtovg", "bench-slint", &[], "windows"),
    ("slint-femtovg overlay", "bench-slint", &[], "overlay"),
];

fn arg(name: &str, default: u64) -> u64 {
    let args: Vec<String> = env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(default)
}

fn main() {
    let (windows, warmup, secs) = (arg("--windows", 4), arg("--warmup", 5), arg("--secs", 20));
    let only = env::args().skip_while(|a| a != "--only").nth(1);
    if !env::args().any(|a| a == "--no-build") {
        let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let ok = Command::new(cargo)
            .args(["build", "--release", "-p", "bench-tauri", "-p", "bench-egui", "-p", "bench-slint"])
            .status()
            .is_ok_and(|s| s.success());
        assert!(ok, "building the variants failed");
    }
    // variants are always built with --release, even when bench itself runs as a debug build
    let dir = env::current_exe().unwrap().parent().unwrap().with_file_name("release");
    let mut sys = System::new_all().expect("read process list");
    sys.refresh_cpu_all();
    let cpu = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default();
    let os = System::long_os_version().unwrap_or_default();
    let mut out = format!(
        "# Widget bench: {os} ({}), {cpu}, {} logical CPUs\n\n{windows} widgets, {warmup}s warmup, {secs}s measured. \
         CPU % is of one core (100 = one full core).\n\n\
         | variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |\n\
         |---|---|---|---|---|---|---|\n",
        env::consts::ARCH,
        sys.cpus().len()
    );
    for (name, bin, args, mode) in VARIANTS {
        if only.as_deref().is_some_and(|o| o != *name) {
            continue;
        }
        eprintln!("== {name}");
        let row = run(&mut sys, &dir, bin, args, mode, windows, warmup, secs);
        out += &format!("| {name} | {row} |\n");
        eprintln!("   {row}");
    }
    print!("\n{out}");
    let file = format!("results-{}-{}.md", env::consts::OS, env::consts::ARCH);
    fs::File::create(&file).and_then(|mut f| f.write_all(out.as_bytes())).expect("write results");
    eprintln!("\nsaved {file}");
}

#[allow(clippy::too_many_arguments)]
fn run(sys: &mut System, dir: &std::path::Path, bin: &str, args: &[&str], mode: &str, windows: u64, warmup: u64, secs: u64) -> String {
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let preexisting = sys.processes().keys().copied().collect();
    let fps = Arc::new(Mutex::new(Vec::<(Instant, f64)>::new()));
    let mut child = (!bin.is_empty()).then(|| {
        let mut c = Command::new(dir.join(format!("{bin}{}", env::consts::EXE_SUFFIX)))
            .args(args)
            .env("BENCH_WINDOWS", windows.to_string())
            .env("BENCH_MODE", mode)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("start {bin}: {e}"));
        let (stdout, fps) = (c.stdout.take().unwrap(), fps.clone());
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(n) = line.strip_prefix("fps ").and_then(|n| n.trim().parse().ok()) {
                    fps.lock().unwrap().push((Instant::now(), n));
                }
            }
        });
        c
    });
    let root = child.as_ref().map(|c| Pid::from_u32(c.id()));
    thread::sleep(Duration::from_secs(warmup));
    sample::sample(sys, root, &preexisting); // primes per-process CPU deltas
    let start = Instant::now();
    let mut samples = vec![];
    while start.elapsed() < Duration::from_secs(secs) {
        let windowserver = if cfg!(target_os = "macos") { sample::windowserver_cpu(sys) } else { None };
        if windowserver.is_none() {
            thread::sleep(Duration::from_secs(1));
        }
        if child.as_mut().is_some_and(|c| c.try_wait().ok().flatten().is_some()) {
            return "exited early | | | | | ".into();
        }
        let mut s = sample::sample(sys, root, &preexisting);
        s.comp_cpu = windowserver.unwrap_or(s.comp_cpu);
        samples.push(s);
    }
    if let (Some(c), Some(root)) = (child.as_mut(), root) {
        sample::kill_tree(sys, root, &preexisting);
        let _ = c.kill();
        let _ = c.wait();
    }
    thread::sleep(Duration::from_secs(2)); // let the compositor settle before the next variant
    let n = samples.len().max(1) as f32;
    let mean = samples.iter().map(|s| s.app_cpu).sum::<f32>() / n;
    let max = samples.iter().map(|s| s.app_cpu).fold(0.0, f32::max);
    let rss = samples.iter().map(|s| s.app_rss).max().unwrap_or(0) as f64 / 1e6;
    let procs = samples.iter().map(|s| s.procs).max().unwrap_or(0);
    let comp = samples.iter().map(|s| s.comp_cpu).sum::<f32>() / n;
    let fps: Vec<f64> = fps.lock().unwrap().iter().filter(|(t, _)| *t >= start).map(|(_, f)| *f).collect();
    let fps = if fps.is_empty() { "-".into() } else { format!("{:.0}", fps.iter().sum::<f64>() / fps.len() as f64) };
    format!("{mean:.1} | {max:.1} | {rss:.0} | {procs} | {comp:.1} | {fps}")
}
