// Runs each widget variant with the same scene and window count, and reports what it costs, how
// fast it starts, and how close it looks to the reference (reference/index.html).
mod capture;
mod look;
mod measure;
mod probe;
mod report;
mod sample;
mod variants;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};
use sysinfo::System;
use variants::{runs_here, VARIANTS};

const COLUMNS: &[&str] = &[
    "app CPU % mean", "app CPU % max", "app RSS MB", "processes", "compositor CPU %", "fps per window",
    "cold start ms", "warm start ms", "widgets found", "placement off px", "look error %",
];

/// First of $BENCH_PYTHON, python3, python that can import PySide6.
fn python() -> Option<String> {
    let candidates = env::var("BENCH_PYTHON").into_iter().chain(["python3".into(), "python".into()]);
    candidates.into_iter().find(|py| {
        Command::new(py).args(["-c", "import PySide6"]).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
    })
}

fn arg(name: &str) -> Option<String> {
    env::args().skip_while(|a| a != name).nth(1)
}

fn flag(name: &str) -> bool {
    env::args().any(|a| a == name)
}

/// reference/ref-<os>-<scale>x.png: `--scale` picks one, else the only one for this OS.
fn reference(dir: &Path) -> Result<(image::RgbaImage, f64), String> {
    let prefix = format!("ref-{}-", env::consts::OS);
    let mut found: Vec<String> = fs::read_dir(dir).into_iter().flatten().flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|f| f.starts_with(&prefix) && f.ends_with("x.png"))
        .collect();
    if let Some(scale) = arg("--scale") {
        found.retain(|f| *f == format!("{prefix}{scale}x.png"));
    }
    let [file] = &found[..] else {
        return Err(format!("{} {prefix}*x.png in reference/; export it from reference/index.html or pass --scale", found.len()));
    };
    let scale = file[prefix.len()..file.len() - 5].parse().map_err(|_| format!("bad scale in {file}"))?;
    let img = image::open(dir.join(file)).map_err(|e| format!("{file}: {e}"))?;
    Ok((img.to_rgba8(), scale))
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let results = root.join("results");
    let prefix = format!("{}-{}-", env::consts::OS, env::consts::ARCH);
    if flag("--summary") {
        let out = report::summary(&results, &prefix);
        print!("{out}");
        fs::write(results.join(format!("{prefix}summary.md")), out).expect("write summary");
        return;
    }
    let num = |name: &str, default: u64| arg(name).and_then(|v| v.parse().ok()).unwrap_or(default);
    let (windows, warmup, secs) = (num("--windows", 4), num("--warmup", 5), num("--secs", 20));
    let only = arg("--only");
    // one package at a time, so a missing system library skips only that package's rows
    let mut failed = vec![];
    if !flag("--no-build") {
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
    let bin_dir = env::current_exe().unwrap().parent().unwrap().with_file_name("release");
    let stamp = report::stamp();
    let shots: PathBuf = results.join(format!("{prefix}{stamp}"));
    fs::create_dir_all(&shots).expect("create results dir");
    let reference = reference(&root.join("reference"));
    let mut sys = System::new_all().expect("read process list");
    sys.refresh_cpu_all();
    let cpu = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default();
    let os_name = System::long_os_version().unwrap_or_default();
    let python = python();
    let qt_script = root.join("qt/main.py");
    let look_note = match &reference {
        Ok((_, scale)) => format!("Look compared with the reference at scale {scale}; pictures in `{prefix}{stamp}/`."),
        Err(e) => format!("Look not compared: {e}."),
    };
    let mut out = format!(
        "# Widget bench: {os_name} ({}), {cpu}, {} logical CPUs, {stamp} UTC\n\n{windows} widgets, {warmup}s warmup, \
         {secs}s measured. CPU % is of one core (100 = one full core). {look_note}\n\n| variant | {} |\n|{}\n",
        env::consts::ARCH,
        sys.cpus().len(),
        COLUMNS.join(" | "),
        "---|".repeat(COLUMNS.len() + 1)
    );
    let skipped = |name: &str, why: &str| format!("| {name} | skipped: {why} |{}\n", " |".repeat(COLUMNS.len() - 1));
    for (name, bin, mode, envs, os) in VARIANTS {
        if only.as_deref().is_some_and(|o| o != *name) {
            continue;
        }
        eprintln!("== {name}");
        let why = if !runs_here(os) {
            Some(format!("{} only", os.join(", ")))
        } else if failed.contains(bin) {
            Some("build failed".into())
        } else if *bin == "qt" && python.is_none() {
            Some("no Python with PySide6 (pip install PySide6)".into())
        } else {
            None
        };
        if let Some(why) = why {
            out += &skipped(name, &why);
            continue;
        }
        let make = || {
            let mut c = match *bin {
                "qt" => Command::new(python.as_ref().unwrap()),
                _ => Command::new(bin_dir.join(format!("{bin}{}", env::consts::EXE_SUFFIX))),
            };
            if *bin == "qt" {
                c.arg(&qt_script);
            }
            c.envs(envs.iter().copied()).env("BENCH_WINDOWS", windows.to_string()).env("BENCH_MODE", mode);
            c
        };
        let row = if bin.is_empty() {
            format!("{} | | | | |", measure::run(&mut sys, None, warmup, secs))
        } else {
            if flag("--purge") && !probe::purge_caches() {
                eprintln!("   --purge: couldn't drop the file cache (run `sudo -v` first); cold start is only a first launch");
            }
            let image = shots.join(format!("{}.png", name.replace(' ', "-")));
            let look = reference.as_ref().ok().map(|(r, scale)| probe::Look {
                reference: r,
                scale: *scale,
                widgets: windows as usize,
                image: &image,
            });
            let probed = probe::probe(&mut sys, &make, look, &shots.join("screenshot.png"));
            format!("{} | {probed}", measure::run(&mut sys, Some(make()), warmup, secs))
        };
        out += &format!("| {name} | {row} |\n");
        eprintln!("   {row}");
    }
    let _ = fs::remove_file(shots.join("screenshot.png"));
    print!("\n{out}");
    if only.is_some() {
        return; // a partial run doesn't go into the results that the summary accumulates
    }
    let file = results.join(format!("{prefix}{stamp}.md"));
    fs::write(&file, out).expect("write results");
    eprintln!("\nsaved {}", file.display());
}
