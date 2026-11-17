// Screenshots of the primary monitor at physical resolution, through each OS's own tool, so the
// bench needs no capture libraries. None if no tool works (e.g. no Screen Recording permission).
use image::RgbaImage;
use std::path::Path;
use std::process::{Command, Stdio};

// Windows: GDI copy of the composed desktop; DPI-aware so a scaled screen isn't captured downscaled,
// CaptureBlt so layered windows are included.
#[cfg(windows)]
const POWERSHELL: &str = r#"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type 'using System.Runtime.InteropServices; public class Dpi { [DllImport("user32.dll")] public static extern bool SetProcessDPIAware(); }'
[Dpi]::SetProcessDPIAware() | Out-Null
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size, [System.Drawing.CopyPixelOperation]'SourceCopy, CaptureBlt')
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
                return Some(img.to_rgba8());
            }
        }
    }
    None
}
