// Finds the widgets in a screenshot and scores how far each strays from the reference look. The
// expected image at a spot is the reference PNG composited over the same spot of a screenshot taken
// before the app started. Pixels that differ between that screenshot and one taken after the app
// closed (a terminal scrolling, a title bar dimming on focus change) are background noise, ignored.
use image::RgbaImage;

/// One expected widget: where it was found (if at all), physical pixels, and its look error.
pub struct Match {
    pub expected: (u32, u32),
    pub found: Option<(u32, u32)>,
    /// 0 = pixel-identical to the reference, 1 = as different as drawing nothing.
    pub error: f64,
}

/// A spot counts as a widget when its coarse (8x8-averaged) error is below this; the averaging
/// forgives small misdrawings, which the full-resolution look error then reports.
const FOUND: f64 = 0.6;
/// Summed RGB difference above which a pixel's background counts as changed during the shot.
const NOISE: u32 = 30;
/// Coarse search on images box-averaged over STEP x STEP pixels, then refined pixel by pixel.
const STEP: u32 = 8;

/// Error of the reference placed with its top-left corner at (x, y): |after - expected| summed,
/// divided by |before - expected| summed.
pub fn error(before: &RgbaImage, after: &RgbaImage, reference: &RgbaImage, x: u32, y: u32) -> f64 {
    let (mut diff, mut base) = (0.0, 0.0);
    for (rx, ry, r) in reference.enumerate_pixels() {
        let (b, s) = (before.get_pixel(x + rx, y + ry), after.get_pixel(x + rx, y + ry));
        if b[3] == 0 {
            continue; // background changed by itself here
        }
        let a = r[3] as f64 / 255.0;
        for c in 0..3 {
            let e = r[c] as f64 * a + b[c] as f64 * (1.0 - a);
            diff += (s[c] as f64 - e).abs();
            base += (b[c] as f64 - e).abs();
        }
    }
    if base == 0.0 { 1.0 } else { diff / base }
}

/// Box-averaged copy, one cell per STEP x STEP pixels, as premultiplied [r·a, g·a, b·a, a] in 0..1.
struct Small {
    w: u32,
    h: u32,
    px: Vec<[f32; 4]>,
}

fn shrink(img: &RgbaImage) -> Small {
    let (w, h) = (img.width() / STEP, img.height() / STEP);
    let mut px = vec![[0.0; 4]; (w * h) as usize];
    for (x, y, p) in img.enumerate_pixels() {
        if x / STEP < w && y / STEP < h {
            let (cell, a) = (&mut px[(y / STEP * w + x / STEP) as usize], p[3] as f32 / 255.0);
            for c in 0..3 {
                cell[c] += p[c] as f32 / 255.0 * a;
            }
            cell[3] += a;
        }
    }
    let n = (STEP * STEP) as f32;
    px.iter_mut().for_each(|c| c.iter_mut().for_each(|v| *v /= n));
    Small { w, h, px }
}

/// `error` on the shrunk images, with the reference at cell (x, y).
fn coarse_error(before: &Small, after: &Small, reference: &Small, x: u32, y: u32) -> f64 {
    let (mut diff, mut base) = (0.0, 0.0);
    for ry in 0..reference.h {
        for rx in 0..reference.w {
            let r = reference.px[(ry * reference.w + rx) as usize];
            let i = ((y + ry) * before.w + x + rx) as usize;
            let (b, s) = (before.px[i], after.px[i]);
            if b[3] < 0.9 {
                continue; // mostly changing background
            }
            for c in 0..3 {
                let e = r[c] + b[c] * (1.0 - r[3]);
                diff += (s[c] - e).abs() as f64;
                base += (b[c] - e).abs() as f64;
            }
        }
    }
    if base == 0.0 { 1.0 } else { diff / base }
}

/// Copies of before and after with alpha 0 where the background changed between before and post.
fn mask(before: &RgbaImage, after: &RgbaImage, post: &RgbaImage) -> (RgbaImage, RgbaImage) {
    let (mut b, mut a) = (before.clone(), after.clone());
    for (x, y, p) in post.enumerate_pixels() {
        let q = before.get_pixel(x, y);
        if (0..3).map(|c| p[c].abs_diff(q[c]) as u32).sum::<u32>() > NOISE {
            b.get_pixel_mut(x, y)[3] = 0;
            a.get_pixel_mut(x, y)[3] = 0;
        }
    }
    (b, a)
}

/// Searches the whole screenshot for `expected.len()` widgets and pairs each hit with the nearest
/// expected position. A widget that is missing gets error 1.
pub fn find(before: &RgbaImage, after: &RgbaImage, post: &RgbaImage, reference: &RgbaImage, expected: &[(u32, u32)]) -> Vec<Match> {
    let (before, after) = &mask(before, after, post);
    let (rw, rh) = reference.dimensions();
    let (maxx, maxy) = (after.width().saturating_sub(rw), after.height().saturating_sub(rh));
    let (b, a, r) = (shrink(before), shrink(after), shrink(reference));
    let mut coarse = vec![];
    for y in 0..=b.h.saturating_sub(r.h) {
        for x in 0..=b.w.saturating_sub(r.w) {
            coarse.push((coarse_error(&b, &a, &r, x, y), x * STEP, y * STEP));
        }
    }
    coarse.sort_by(|a, b| a.0.total_cmp(&b.0));
    // best non-overlapping hits, each refined to the exact pixel
    let mut hits: Vec<(f64, u32, u32)> = vec![];
    for &(e, x, y) in &coarse {
        if hits.len() == expected.len() || e >= FOUND {
            break;
        }
        if hits.iter().any(|h| h.1.abs_diff(x) < rw && h.2.abs_diff(y) < rh) {
            continue;
        }
        let mut best = (f64::MAX, x, y);
        for ny in y.saturating_sub(STEP)..=(y + STEP).min(maxy) {
            for nx in x.saturating_sub(STEP)..=(x + STEP).min(maxx) {
                let e = error(before, after, reference, nx, ny);
                if e < best.0 {
                    best = (e, nx, ny);
                }
            }
        }
        hits.push(best);
    }
    let mut matches: Vec<Match> = expected.iter().map(|&p| Match { expected: p, found: None, error: 1.0 }).collect();
    let dist = |m: &Match, h: &(f64, u32, u32)| m.expected.0.abs_diff(h.1).pow(2) + m.expected.1.abs_diff(h.2).pow(2);
    for h in &hits {
        if let Some(m) = matches.iter_mut().filter(|m| m.found.is_none()).min_by_key(|m| dist(m, h)) {
            (m.found, m.error) = (Some((h.1, h.2)), h.0);
        }
    }
    matches
}

/// Before-with-reference (what it should look like) above the actual screenshot, cropped to the
/// area around the expected and found widgets, for eyeballing.
pub fn side_by_side(before: &RgbaImage, after: &RgbaImage, reference: &RgbaImage, matches: &[Match]) -> RgbaImage {
    let (rw, rh) = reference.dimensions();
    let spots = matches.iter().flat_map(|m| std::iter::once(m.expected).chain(m.found));
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y) in spots {
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + rw), y1.max(y + rh));
    }
    let pad = rh / 4;
    let (x0, y0) = (x0.saturating_sub(pad), y0.saturating_sub(pad));
    let (x1, y1) = ((x1 + pad).min(after.width()), (y1 + pad).min(after.height()));
    let (w, h) = (x1 - x0, y1 - y0);
    let mut out = RgbaImage::new(w, h * 2);
    for y in 0..h {
        for x in 0..w {
            let mut e = *before.get_pixel(x0 + x, y0 + y);
            for m in matches {
                let (mx, my) = m.expected;
                if (mx..mx + rw).contains(&(x0 + x)) && (my..my + rh).contains(&(y0 + y)) {
                    let r = reference.get_pixel(x0 + x - mx, y0 + y - my);
                    let a = r[3] as f64 / 255.0;
                    for c in 0..3 {
                        e[c] = (r[c] as f64 * a + e[c] as f64 * (1.0 - a)).round() as u8;
                    }
                }
            }
            out.put_pixel(x, y, e);
            out.put_pixel(x, h + y, *after.get_pixel(x0 + x, y0 + y));
        }
    }
    out
}

#[cfg(test)]
#[path = "look_test.rs"]
mod test;
