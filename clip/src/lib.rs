//! Shapes an overlay window to its widget rectangles: clicks outside them fall through to any app
//! below, while the widgets stay clickable. Windows only (SetWindowRgn); a no-op elsewhere.

/// `hwnd` is the raw Win32 window handle; `rects` are (x, y, width, height) in physical pixels.
#[cfg(windows)]
pub fn to_rects(hwnd: *mut std::ffi::c_void, rects: &[(i32, i32, i32, i32)]) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, DeleteObject, SetWindowRgn, RGN_OR};
    unsafe {
        let region = CreateRectRgn(0, 0, 0, 0);
        for &(x, y, w, h) in rects {
            let rect = CreateRectRgn(x, y, x + w, y + h);
            CombineRgn(Some(region), Some(region), Some(rect), RGN_OR);
            let _ = DeleteObject(rect.into());
        }
        SetWindowRgn(HWND(hwnd), Some(region), true); // the system owns the region from here
    }
}

/// Physical-pixel rectangles of the bench's widget grid: `n` panels of 240x110 at `scale`. The top
/// row is at y = 60 so windows (8 px larger for the glow) clear the ~37 pt macOS menu bar.
pub fn panels(n: usize, scale: f64) -> Vec<(i32, i32, i32, i32)> {
    let px = |v: f64| (v * scale).round() as i32;
    (0..n)
        .map(|i| {
            let (x, y) = (40.0 + 260.0 * (i % 4) as f64, 60.0 + 140.0 * (i / 4) as f64);
            (px(x), px(y), px(x + 240.0) - px(x), px(y + 110.0) - px(y))
        })
        .collect()
}

#[test]
fn panels_scale_to_physical_pixels() {
    assert_eq!(panels(5, 1.5), vec![
        (60, 90, 360, 165), (450, 90, 360, 165), (840, 90, 360, 165), (1230, 90, 360, 165), (60, 300, 360, 165),
    ]);
}
