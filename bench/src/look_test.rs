use super::*;
use image::Rgba;

/// Noisy background, a half-transparent 40x20 "widget" with a pattern, three of four widgets
/// drawn (one 30 px off), one missing.
#[test]
fn finds_moved_and_missing_widgets() {
    let mut seed = 1u32;
    let mut noise = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 24) as u8
    };
    let before = RgbaImage::from_fn(400, 200, |_, _| Rgba([noise(), noise(), noise(), 255]));
    let reference = RgbaImage::from_fn(40, 20, |x, y| {
        if (x / 4 + y / 4) % 2 == 0 { Rgba([80, 255, 120, 200]) } else { Rgba([3, 10, 3, 120]) }
    });
    let expected = [(10, 10), (100, 10), (200, 10), (300, 10)];
    let drawn = [(10, 10), (130, 40), (200, 10)]; // second moved by (30, 30), fourth missing
    let mut after = before.clone();
    for &(dx, dy) in &drawn {
        for (x, y, r) in reference.enumerate_pixels() {
            let b = after.get_pixel_mut(dx + x, dy + y);
            let a = r[3] as f64 / 255.0;
            for c in 0..3 {
                b[c] = (r[c] as f64 * a + b[c] as f64 * (1.0 - a)).round() as u8;
            }
        }
    }
    // the background changed by itself in one strip (say, a terminal scrolled): ignored
    let mut post = before.clone();
    for x in 0..400 {
        post.put_pixel(x, 15, Rgba([0, 0, 0, 255])); // crosses widget 0
    }
    let m = find(&before, &after, &post, &reference, &expected);
    assert_eq!(m[0].found, Some((10, 10)));
    assert_eq!(m[1].found, Some((130, 40)));
    assert_eq!(m[2].found, Some((200, 10)));
    assert_eq!(m[3].found, None);
    assert!(m[0].error < 0.01 && m[1].error < 0.01, "errors {} {}", m[0].error, m[1].error);
    assert_eq!(m[3].error, 1.0);
    let img = side_by_side(&before, &after, &reference, &m);
    assert_eq!(img.height() % 2, 0);
}
