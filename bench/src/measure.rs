// The steady-state measurement: CPU, memory, compositor CPU and fps while the widgets animate.
use crate::sample;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

/// Cells: app CPU % mean | max | RSS MB | processes | compositor CPU % | fps per window
pub fn run(sys: &mut System, cmd: Option<Command>, warmup: u64, secs: u64) -> String {
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let preexisting = sys.processes().keys().copied().collect();
    let fps = Arc::new(Mutex::new(Vec::<(Instant, f64)>::new()));
    let mut child = cmd.map(|mut cmd| {
        let mut c = cmd
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
