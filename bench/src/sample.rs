// Attributes CPU and memory to a launched app: its process tree, plus helpers it does not parent.
use std::collections::HashSet;
use sysinfo::{Pid, ProcessesToUpdate, System};

// Desktop compositors: transparent windows cost them work too, which the app's own CPU hides.
const COMPOSITORS: &[&str] = &[
    "WindowServer", "dwm.exe", "gnome-shell", "kwin_wayland", "kwin_x11", "Xorg", "Xwayland",
    "sway", "Hyprland", "picom", "mutter", "weston", "labwc", "niri",
];

pub struct Sample {
    pub app_cpu: f32,
    pub app_rss: u64,
    pub procs: usize,
    pub comp_cpu: f32,
}

/// Processes only: on Linux sysinfo also lists every thread, each reporting its whole process's
/// memory and its own share of the CPU, which multiplied RSS (45 GB for tauri) and double-counted CPU.
pub fn app_pids(sys: &System, root: Option<Pid>, preexisting: &HashSet<Pid>) -> HashSet<Pid> {
    let mut app = HashSet::new();
    let Some(root) = root else { return app };
    app.insert(root);
    loop {
        let before = app.len();
        for (pid, p) in sys.processes() {
            if p.thread_kind().is_none() && p.parent().is_some_and(|pp| app.contains(&pp)) {
                app.insert(*pid);
            }
        }
        if app.len() == before {
            break;
        }
    }
    // macOS WKWebView helpers are spawned by launchd, not the app: count the ones born during this run.
    for (pid, p) in sys.processes() {
        if !preexisting.contains(pid) && p.name().to_string_lossy().starts_with("com.apple.WebKit") {
            app.insert(*pid);
        }
    }
    app
}

pub fn sample(sys: &mut System, root: Option<Pid>, preexisting: &HashSet<Pid>) -> Sample {
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let app = app_pids(sys, root, preexisting);
    let mut s = Sample { app_cpu: 0.0, app_rss: 0, procs: app.len(), comp_cpu: 0.0 };
    for (pid, p) in sys.processes() {
        if p.thread_kind().is_some() {
            continue; // a thread of a process already counted (Linux)
        }
        if app.contains(pid) {
            s.app_cpu += p.cpu_usage();
            s.app_rss += p.memory();
        } else if COMPOSITORS.contains(&&*p.name().to_string_lossy()) {
            s.comp_cpu += p.cpu_usage();
        }
    }
    s
}

pub fn kill_tree(sys: &mut System, root: Pid, preexisting: &HashSet<Pid>) {
    sys.refresh_processes(ProcessesToUpdate::All, true);
    for pid in app_pids(sys, Some(root), preexisting) {
        if let Some(p) = sys.process(pid) {
            p.kill();
        }
    }
}

/// macOS hides WindowServer's CPU from non-root processes, but `top` may read it. Blocks ~1.2s,
/// so it stands in for the sampling sleep. None elsewhere, or if WindowServer is not found.
pub fn windowserver_cpu(sys: &System) -> Option<f32> {
    let (pid, _) = sys.processes().iter().find(|(_, p)| p.name() == "WindowServer")?;
    let out = std::process::Command::new("top")
        .args(["-l", "2", "-s", "1", "-stats", "cpu", "-pid", &pid.to_string()])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).lines().last()?.trim().parse().ok()
}

#[test]
fn threads_are_not_processes() {
    let _thread = std::thread::spawn(|| std::thread::sleep(std::time::Duration::from_secs(2)));
    let mut sys = System::new().expect("process list");
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let me = sysinfo::get_current_pid().unwrap();
    assert_eq!(app_pids(&sys, Some(me), &HashSet::new()).len(), 1, "only this process, not its threads");
}
