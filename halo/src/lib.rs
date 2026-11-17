//! The reference look's glows (tauri/dist/widget.js) for renderers without a blur: egui, Slint and
//! GTK/Cairo. Each glow is rendered once, at the screen's scale, the way canvas 2D draws a shadow:
//! the shape's alpha, Gaussian-blurred with sigma = shadowBlur / 2, times the shadow color. The
//! renderer draws the images at the same depth as the reference and everything else natively.
mod blur;

use ab_glyph::{Font, FontRef, ScaleFont};
use tiny_skia::{FillRule, Mask, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub const W: f32 = 240.0;
pub const H: f32 = 110.0;
/// Room around the panel for the outer glow; windows are this much larger on every side.
pub const MARGIN: f32 = 8.0;
pub const FONT: &[u8] = include_bytes!("../../fonts/SpaceMono-Regular.ttf");
pub const FONT_SIZE: f32 = 12.0;
/// Space Mono at 12 px: ascent (1120/1000 em), the baseline's distance below a text box's top.
pub const ASCENT: f32 = 13.44;
/// Every glyph's advance (612/1000 em): text widths for centering.
pub const ADVANCE: f32 = 7.344;
pub const TITLE: (f32, f32) = (10.0, 20.0); // baseline start of "MOTORS"
pub const NUMBER_Y: f32 = 94.0; // baseline of the numbers, centered on each pie
pub const R: f32 = 20.0;
pub const PIE_Y: f32 = 50.0;

pub fn pie_x(i: usize) -> f32 {
    32.0 + 52.0 * i as f32
}

/// Fake PWM of motor `i` at time `t`, as in the reference.
pub fn pwm(t: f32, i: usize) -> f32 {
    0.5 + 0.45 * (t * 1.3 + i as f32).sin()
}

/// A glow image: premultiplied RGBA, `width` x `height` physical pixels, whose top-left corner
/// sits at (`left`, `top`) logical px from the panel's (or, for digits, the glyph's pen) origin.
pub struct Sprite {
    pub left: f32,
    pub top: f32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Canvas-style rounded rectangle path.
pub fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> tiny_skia::Path {
    const K: f32 = 0.552_284_8; // cubic approximation of a quarter circle
    let mut p = PathBuilder::new();
    p.move_to(x + r, y);
    p.line_to(x + w - r, y);
    p.cubic_to(x + w - r + K * r, y, x + w, y + r - K * r, x + w, y + r);
    p.line_to(x + w, y + h - r);
    p.cubic_to(x + w, y + h - r + K * r, x + w - r + K * r, y + h, x + w - r, y + h);
    p.line_to(x + r, y + h);
    p.cubic_to(x + r - K * r, y + h, x, y + h - r + K * r, x, y + h - r);
    p.line_to(x, y + r);
    p.cubic_to(x, y + r - K * r, x + r - K * r, y, x + r, y);
    p.close();
    p.finish().unwrap()
}

/// Canvas over a logical area (x0, y0, w, h) at `scale`; returns the pixmap and the transform
/// that maps logical panel coordinates into it.
pub fn canvas(x0: f32, y0: f32, w: f32, h: f32, scale: f32) -> (Pixmap, Transform) {
    let pixmap = Pixmap::new((w * scale).ceil() as u32, (h * scale).ceil() as u32).unwrap();
    (pixmap, Transform::from_translate(-x0, -y0).post_scale(scale, scale))
}

fn paint(alpha: f32) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(tiny_skia::Color::from_rgba(1.0, 1.0, 1.0, alpha).unwrap());
    p
}

/// Blurs the pixmap's alpha and tints it with the shadow color: the sprite.
fn shadow(pixmap: &Pixmap, x0: f32, y0: f32, scale: f32, sigma: f32, color: [f32; 4]) -> Sprite {
    let (w, h) = (pixmap.width(), pixmap.height());
    let mut a: Vec<f32> = pixmap.pixels().iter().map(|p| p.alpha() as f32 / 255.0).collect();
    blur::gaussian(&mut a, w as usize, h as usize, sigma * scale);
    let mut rgba = Vec::with_capacity(a.len() * 4);
    for a in a {
        let k = (a * color[3]).clamp(0.0, 1.0);
        rgba.extend([color[0] * k, color[1] * k, color[2] * k, k].map(|v| (v * 255.0).round() as u8));
    }
    Sprite { left: x0, top: y0, width: w, height: h, rgba }
}

const ACCENT: [f32; 3] = [80.0 / 255.0, 1.0, 120.0 / 255.0];

/// Outer glow: the 1 px border (alpha 0.35) blurred, shadowBlur 6, color alpha 0.45. Drawn first.
pub fn outer(scale: f32) -> Sprite {
    let (x0, y0) = (-MARGIN - 2.0, -MARGIN - 2.0);
    let (mut px, t) = canvas(x0, y0, W - 2.0 * x0, H - 2.0 * y0, scale);
    let stroke = Stroke { width: 1.0, ..Default::default() };
    px.stroke_path(&rounded(0.5, 0.5, W - 1.0, H - 1.0, 4.0), &paint(0.35), &stroke, t, None);
    shadow(&px, x0, y0, scale, 3.0, [ACCENT[0], ACCENT[1], ACCENT[2], 0.45])
}

/// Inner glow: a 2 px band just outside the panel (alpha 0.15) blurred, shadowBlur 24, color
/// alpha 0.5, clipped to the panel. Drawn over the grid, under the second border.
pub fn inner(scale: f32) -> Sprite {
    let (px, t) = canvas(0.0, 0.0, W, H, scale);
    let stroke = Stroke { width: 2.0, ..Default::default() };
    // the band lies outside this canvas; draw it on a padded one, blur there, then crop
    let pad = 40.0;
    let (mut big, bt) = canvas(-pad, -pad, W + 2.0 * pad, H + 2.0 * pad, scale);
    big.stroke_path(&rounded(-1.0, -1.0, W + 2.0, H + 2.0, 4.0), &paint(0.15), &stroke, bt, None);
    let glow = shadow(&big, -pad, -pad, scale, 12.0, [ACCENT[0], ACCENT[1], ACCENT[2], 0.5]);
    let mut clip = Mask::new(px.width(), px.height()).unwrap();
    clip.fill_path(&rounded(0.5, 0.5, W - 1.0, H - 1.0, 4.0), FillRule::Winding, true, t);
    let (off, bw) = ((pad * scale).round() as u32, glow.width);
    let mut rgba = Vec::with_capacity((px.width() * px.height() * 4) as usize);
    for y in 0..px.height() {
        for x in 0..px.width() {
            let i = (((y + off) * bw + x + off) * 4) as usize;
            let m = clip.data()[(y * px.width() + x) as usize] as f32 / 255.0;
            rgba.extend(glow.rgba[i..i + 4].iter().map(|v| (*v as f32 * m).round() as u8));
        }
    }
    Sprite { left: 0.0, top: 0.0, width: px.width(), height: px.height(), rgba }
}

/// Coverage of `text` with its baseline starting at (x, y), drawn into `px` (logical coords via t).
pub fn text(px: &mut Pixmap, text: &str, x: f32, y: f32, scale: f32, t: Transform) {
    let font = FontRef::try_from_slice(FONT).unwrap();
    // ab_glyph scales by ascent - descent, not by the em: convert the em size in pixels
    let em = FONT_SIZE * scale * font.height_unscaled() / font.units_per_em().unwrap();
    let sized = font.as_scaled(ab_glyph::PxScale::from(em));
    let (w, h) = (px.width() as i32, px.height() as i32);
    let mut pen = x;
    for c in text.chars() {
        let (gx, gy) = (t.sx * pen + t.tx, t.sy * y + t.ty);
        let glyph = sized.scaled_glyph(c);
        let glyph = ab_glyph::Glyph { position: ab_glyph::point(gx, gy), ..glyph };
        if let Some(outline) = font.outline_glyph(glyph) {
            let b = outline.px_bounds();
            let data = px.data_mut();
            outline.draw(|ox, oy, cov| {
                let (px_x, px_y) = (b.min.x as i32 + ox as i32, b.min.y as i32 + oy as i32);
                if (0..w).contains(&px_x) && (0..h).contains(&px_y) {
                    let i = ((px_y * w + px_x) * 4) as usize;
                    let a = ((cov * 255.0).round() as u8).max(data[i + 3]);
                    data[i..i + 4].copy_from_slice(&[a, a, a, a]);
                }
            });
        }
        pen += ADVANCE;
    }
}

/// Title glow: "MOTORS" blurred, shadowBlur 6, color = accent at full alpha. Drawn under the title.
pub fn title(scale: f32) -> Sprite {
    let (x0, y0) = (0.0, 0.0);
    let (mut px, t) = canvas(x0, y0, 70.0, 34.0, scale);
    text(&mut px, "MOTORS", TITLE.0, TITLE.1, scale, t);
    shadow(&px, x0, y0, scale, 3.0, [ACCENT[0], ACCENT[1], ACCENT[2], 1.0])
}

/// Glow of each digit 0-9 with its pen at (0, 0) on the baseline: shadowBlur 8, color alpha 0.6.
/// A number's glow is its digits' glows, one ADVANCE apart (blur is linear; glyphs don't overlap).
pub fn digits(scale: f32) -> Vec<Sprite> {
    (0..10)
        .map(|d| {
            let (x0, y0) = (-14.0, -28.0);
            let (mut px, t) = canvas(x0, y0, ADVANCE + 28.0, 40.0, scale);
            text(&mut px, &d.to_string(), 0.0, 0.0, scale, t);
            shadow(&px, x0, y0, scale, 4.0, [ACCENT[0], ACCENT[1], ACCENT[2], 0.6])
        })
        .collect()
}

/// Left edge of a number's first glyph when centered on `cx`, as canvas textAlign "center" does.
pub fn centered(cx: f32, text: &str) -> f32 {
    cx - ADVANCE * text.chars().count() as f32 / 2.0
}

#[cfg(test)]
mod test;
