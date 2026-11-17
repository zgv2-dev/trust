// Runs each widget variant with the same scene and window count, and reports what it costs.
mod sample;
mod variants;

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, fs, thread};
use sysinfo::{Pid, System};
use variants::{runs_here, VARIANTS};

/// First of $BENCH_PYTHON, python3, python that can import PySide6.
fn python() -> Option<String> {
    let candidates = env::var("BENCH_PYTHON").into_iter().chain(["python3".into(), "python".into()]);
    candidates.into_iter().find(|py| {
        Command::new(py).args(["-c", "import PySide6"]).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
    })
}

fn arg(name: &str, default: u64) -> u64 {
    let args: Vec<String> = env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(default)
}

fn main() {
    let (windows, warmup, secs) = (arg("--windows", 4), arg("--warmup", 5), arg("--secs", 20));
    let only = env::args().skip_while(|a| a != "--only").nth(1);
    // one package at a time, so a missing system library skips only that package's rows
    let mut failed = vec![];
    if !env::args().any(|a| a == "--no-build") {
        let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let mut packages: Vec<&str> =
            VARIANTS.iter().filter(|v| runs_here(v.4) && v.1.starts_with("bench-")).map(|v| v.1).collect();
        packages.dedup();
        for p in packages {
            if !Command::new(&cargo).args(["build", "--release", "-p", p]).status().is_ok_and(|s| s.success()) {
                eprintln!("building {p} failed; its rows will be skipped");
                failed.push(p);
            }
        }
    }
    // variants are always built with --release, even when bench itself runs as a debug build
    let dir = env::current_exe().unwrap().parent().unwrap().with_file_name("release");
    let mut sys = System::new_all().expect("read process list");
    sys.refresh_cpu_all();
    let cpu = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default();
    let os = System::long_os_version().unwrap_or_default();
    let python = python();
    let qt_script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../qt/main.py");
    let mut out = format!(
        "# Widget bench: {os} ({}), {cpu}, {} logical CPUs\n\n{windows} widgets, {warmup}s warmup, {secs}s measured. \
         CPU % is of one core (100 = one full core).\n\n\
         | variant | app CPU % mean | app CPU % max | app RSS MB | processes | compositor CPU % | fps per window |\n\
         |---|---|---|---|---|---|---|\n",
        env::consts::ARCH,
        sys.cpus().len()
    );
    for (name, bin, mode, envs, os) in VARIANTS {
        if only.as_deref().is_some_and(|o| o != *name) {
            continue;
        }
        eprintln!("== {name}");
        let cmd = match (*bin, &python) {
            ("", _) => None,
            ("qt", Some(py)) => Some(Command::new(py)).map(|mut c| { c.arg(&qt_script); c }),
            _ if !runs_here(os) => {
                out += &format!("| {name} | skipped: {} only | | | | | |\n", os.join(", "));
                continue;
            }
            _ if failed.contains(bin) => {
                out += &format!("| {name} | skipped: build failed | | | | | |\n");
                continue;
            }
            ("qt", None) => {
                out += &format!("| {name} | skipped: no Python with PySide6 (pip install PySide6) | | | | | |\n");
                continue;
            }
            _ => Some(Command::new(dir.join(format!("{bin}{}", env::consts::EXE_SUFFIX)))),
        };
        let row = run(&mut sys, cmd, mode, envs, windows, warmup, secs);
        out += &format!("| {name} | {row} |\n");
        eprintln!("   {row}");
    }
    print!("\n{out}");
    if only.is_some() {
        return; // a partial run must not overwrite a full results file
    }
    let file = format!("results-{}-{}.md", env::consts::OS, env::consts::ARCH);
    fs::File::create(&file).and_then(|mut f| f.write_all(out.as_bytes())).expect("write results");
    eprintln!("\nsaved {file}");
}

fn run(sys: &mut System, cmd: Option<Command>, mode: &str, envs: &[(&str, &str)], windows: u64, warmup: u64, secs: u64) -> String {
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let preexisting = sys.processes().keys().copied().collect();
    let fps = Arc::new(Mutex::new(Vec::<(Instant, f64)>::new()));
    let mut child = cmd.map(|mut cmd| {
        let mut c = cmd
            .envs(envs.iter().copied())
            .env("BENCH_WINDOWS", windows.to_string())
            .env("BENCH_MODE", mode)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("start {:?}: {e}", cmd.get_program()));
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
