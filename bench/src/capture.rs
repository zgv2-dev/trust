// Screenshots of the primary monitor at physical resolution, through each OS's own tool, so the
// bench needs no capture libraries. None if no tool works (e.g. no Screen Recording permission).
use image::RgbaImage;
use std::path::Path;
use std::process::{Command, Stdio};

// Windows: GDI copy of the composed desktop; DPI-aware so a scaled screen isn't captured downscaled,
// SRCCOPY|CAPTUREBLT so layered windows are included (through BitBlt itself: Graphics.CopyFromScreen
// throws InvalidEnumArgumentException for any combined CopyPixelOperation), 24-bit so there's no
// alpha channel, and any error stops the script instead of saving the untouched blank bitmap.
#[cfg(windows)]
const POWERSHELL: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices;
public class Gdi {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr hwnd, IntPtr dc);
  [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr dst, int x, int y, int w, int h, IntPtr src, int sx, int sy, uint rop);
}
'@
[Gdi]::SetProcessDPIAware() | Out-Null
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height, ([System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dst = $g.GetHdc(); $src = [Gdi]::GetDC([IntPtr]::Zero)
$ok = [Gdi]::BitBlt($dst, 0, 0, $b.Width, $b.Height, $src, $b.X, $b.Y, 0x40CC0020) # SRCCOPY | CAPTUREBLT
[Gdi]::ReleaseDC([IntPtr]::Zero, $src) | Out-Null; $g.ReleaseHdc($dst)
if (-not $ok) { throw "BitBlt failed" }
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

// GNOME on Wayland allows screenshots only through the desktop portal: ask it (non-interactive;
// GNOME may ask once whether to allow it), wait for its Response signal, move the file it saved.
#[cfg(not(any(windows, target_os = "macos")))]
const PORTAL: &str = r#"
import sys, shutil, urllib.parse
from gi.repository import Gio, GLib
bus = Gio.bus_get_sync(Gio.BusType.SESSION)
token = "bench"
sender = bus.get_unique_name()[1:].replace(".", "_")
request = f"/org/freedesktop/portal/desktop/request/{sender}/{token}"
loop, result = GLib.MainLoop(), {}
def done(_c, _s, _p, _i, _n, params):
    code, res = params.unpack()
    result.update(res if code == 0 else {}); loop.quit()
bus.signal_subscribe(None, "org.freedesktop.portal.Request", "Response", request, None, 0, done)
bus.call_sync("org.freedesktop.portal.Desktop", "/org/freedesktop/portal/desktop",
    "org.freedesktop.portal.Screenshot", "Screenshot",
    GLib.Variant("(sa{sv})", ("", {"handle_token": GLib.Variant("s", token), "interactive": GLib.Variant("b", False)})),
    None, 0, -1, None)
GLib.timeout_add_seconds(30, loop.quit)
loop.run()
if "uri" not in result:
    sys.exit("portal screenshot refused or timed out")
shutil.move(urllib.parse.unquote(urllib.parse.urlparse(result["uri"]).path), sys.argv[1])
"#;

// first that works: wlroots/Hyprland, KDE, GNOME (portal), then X11 (ImageMagick)
#[cfg(not(any(windows, target_os = "macos")))]
fn tools(path: &str) -> Vec<Command> {
    vec![
        cmd("grim", &[path]),
        cmd("spectacle", &["-b", "-n", "-m", "-o", path]),
        cmd("python3", &["-c", PORTAL, path]),
        cmd("import", &["-window", "root", path]),
    ]
}

pub fn screenshot(path: &Path) -> Option<RgbaImage> {
    let p = path.to_string_lossy();
    let mut error = String::new();
    for mut tool in tools(&p) {
        let _ = std::fs::remove_file(path);
        let out = tool.stdout(Stdio::null()).stderr(Stdio::piped()).output();
        let ok = out.as_ref().is_ok_and(|o| o.status.success());
        error = match &out {
            Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_string(),
            Err(e) => format!("{:?}: {e}", tool.get_program()),
        };
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
    // say why once: a silent failure leaves every look column at "-"
    static WHY: std::sync::Once = std::sync::Once::new();
    WHY.call_once(|| eprintln!("   screenshot failed, look not compared: {}", error.lines().last().unwrap_or("")));
    None
}
