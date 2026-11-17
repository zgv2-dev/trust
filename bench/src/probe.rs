// Startup time and look: one cold launch with the animation frozen, screenshotted and compared
// with the reference, then a few warm launches timed to their first frame.
use crate::{capture, look, report::median, sample};
use image::RgbaImage;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

/// Must match FREEZE in tauri/dist/widget.js, which the reference PNGs are exported at.
const FREEZE: &str = "2";
const WARM_RUNS: usize = 3;

pub struct Look<'a> {
    pub reference: &'a RgbaImage,
    pub scale: f64,
    pub widgets: usize,
    pub image: &'a Path, // where the expected-vs-actual picture goes
}

struct Running {
    child: Child,
    preexisting: HashSet<Pid>,
}

/// Starts the app; returns ms until it printed "first-frame" (None if not within 30 s).
fn launch(sys: &mut System, mut cmd: Command) -> (Option<f64>, Running) {
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let preexisting = sys.processes().keys().copied().collect();
    let start = Instant::now();
    let mut child = cmd.stdout(Stdio::piped()).spawn().unwrap_or_else(|e| panic!("start {:?}: {e}", cmd.get_program()));
    let (tx, rx) = mpsc::channel();
    let stdout = child.stdout.take().unwrap();
    std::thread::spawn(move || {
        // keep reading to the end, or the app blocks once the pipe fills
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if line.trim() == "first-frame" {
                let _ = tx.send(());
            }
        }
    });
    let ms = rx.recv_timeout(Duration::from_secs(30)).ok().map(|_| start.elapsed().as_secs_f64() * 1000.0);
    (ms, Running { child, preexisting })
}

fn stop(sys: &mut System, mut r: Running) {
    sample::kill_tree(sys, Pid::from_u32(r.child.id()), &r.preexisting);
    let _ = r.child.kill();
    let _ = r.child.wait();
    std::thread::sleep(Duration::from_secs(1));
}

/// Drops the OS file cache so the next launch is really cold. Needs passwordless or cached sudo
/// (run `sudo -v` first); Windows has no equivalent.
pub fn purge_caches() -> bool {
    let args: &[&str] = if cfg!(target_os = "macos") {
        &["-n", "purge"]
    } else if cfg!(target_os = "linux") {
        &["-n", "sh", "-c", "sync; echo 3 > /proc/sys/vm/drop_caches"]
    } else {
        return false;
    };
    Command::new("sudo").args(args).status().is_ok_and(|s| s.success())
}

/// Cells: cold start ms | warm start ms | widgets found | placement off px | look error %
pub fn probe(sys: &mut System, make: &dyn Fn() -> Command, look: Option<Look>, shot: &Path) -> String {
    let before = look.as_ref().and_then(|_| capture::screenshot(shot));
    let mut cmd = make();
    cmd.env("BENCH_FREEZE", FREEZE);
    let (cold, running) = launch(sys, cmd);
    std::thread::sleep(Duration::from_secs(3)); // let windows settle: overlays resize, WMs place
    let after = before.as_ref().and_then(|_| capture::screenshot(shot));
    stop(sys, running);
    let mut warm: Vec<f64> = (0..WARM_RUNS)
        .filter_map(|_| {
            let (ms, r) = launch(sys, make());
            stop(sys, r);
            ms
        })
        .collect();
    let ms = |v: Option<f64>| v.map_or("no first frame".into(), |v| format!("{v:.0}"));
    let times = format!("{} | {}", ms(cold), ms(median(&mut warm)));
    let (Some(look), Some(before), Some(after)) = (look, before, after) else {
        return format!("{times} | - | - | -");
    };
    let px = |v: f64| (v * look.scale).round() as u32;
    let expected: Vec<(u32, u32)> = (0..look.widgets)
        .map(|i| (px(40.0 + 260.0 * (i % 4) as f64), px(40.0 + 140.0 * (i / 4) as f64)))
        .collect();
    let matches = look::find(&before, &after, look.reference, &expected);
    let _ = look::side_by_side(&before, &after, look.reference, &matches).save(look.image);
    let found: Vec<f64> = matches
        .iter()
        .filter_map(|m| m.found.map(|f| m.expected.0.abs_diff(f.0).pow(2) as f64 + m.expected.1.abs_diff(f.1).pow(2) as f64))
        .collect();
    let off = found.iter().map(|d| d.sqrt() / look.scale).fold(None, |a: Option<f64>, d| Some(a.map_or(d, |a| a.max(d))));
    let error = matches.iter().map(|m| m.error).sum::<f64>() / matches.len().max(1) as f64 * 100.0;
    let off = off.map_or("-".into(), |o| format!("{o:.0}"));
    format!("{times} | {}/{} | {off} | {error:.1}", found.len(), matches.len())
}
