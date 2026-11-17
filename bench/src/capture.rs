// Screenshots of the primary monitor at physical resolution, through each OS's own tool, so the
// bench needs no capture libraries. None if no tool works (e.g. no Screen Recording permission).
use image::RgbaImage;
use std::path::Path;
use std::process::{Command, Stdio};

// Windows: GDI copy of the composed desktop; DPI-aware so a scaled screen isn't captured downscaled,
// CaptureBlt so layered windows are included (CopyPixelOperation isn't a flags enum, so the two are
// combined numerically), 24-bit so there's no alpha channel, and any error stops the script
// instead of saving the untouched blank bitmap.
#[cfg(windows)]
const POWERSHELL: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type 'using System.Runtime.InteropServices; public class Dpi { [DllImport("user32.dll")] public static extern bool SetProcessDPIAware(); }'
[Dpi]::SetProcessDPIAware() | Out-Null
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height, ([System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$op = [int][System.Drawing.CopyPixelOperation]::SourceCopy -bor [int][System.Drawing.CopyPixelOperation]::CaptureBlt
$g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size, [System.Drawing.CopyPixelOperation]$op)
$bmp.Save($env:BENCH_SHOT, [System.Drawing.Imaging.ImageFormat]::Png)
"#;

fn cmd(program: &str, args: &[&str]) -> Command {
    let mut c = Command::new(program);
    c.args(args);
    c
}

#[cfg(target_os = "macos")]
fn tools(path: &str) -> Vec<Command> {
    vec![cmd("screencapture", &["-x", "-m", "-t", "png", path])]
}

#[cfg(windows)]
fn tools(path: &str) -> Vec<Command> {
    let mut c = cmd("powershell", &["-NoProfile", "-NonInteractive", "-Command", POWERSHELL]);
    c.env("BENCH_SHOT", path);
    vec![c]
}

// first that works: wlroots/Hyprland, KDE, GNOME, then X11 (ImageMagick)
#[cfg(not(any(windows, target_os = "macos")))]
fn tools(path: &str) -> Vec<Command> {
    vec![
        cmd("grim", &[path]),
        cmd("spectacle", &["-b", "-n", "-m", "-o", path]),
        cmd("gnome-screenshot", &["-f", path]),
        cmd("import", &["-window", "root", path]),
    ]
}

pub fn screenshot(path: &Path) -> Option<RgbaImage> {
    let p = path.to_string_lossy();
    for mut tool in tools(&p) {
        let _ = std::fs::remove_file(path);
        let ok = tool.stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success());
        if ok {
            if let Ok(img) = image::open(path) {
                // a screen has no transparency; some captures leave alpha 0, which would read as
                // "background changed" everywhere and hide every widget from the look check
                let mut img = img.to_rgba8();
                img.pixels_mut().for_each(|p| p[3] = 255);
                return Some(img);
            }
        }
    }
    None
}
