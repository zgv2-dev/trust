// Paints one panel at t = 2 entirely on the CPU (tiny-skia shapes, ab_glyph text, halo glows) in
// the reference's order, to check halo against the reference PNG without any GUI toolkit:
// cargo run -p halo --example preview -- <scale> <out.png>
use halo::*;
use tiny_skia::{Color, FillRule, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

fn paint(a: f32) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(Color::from_rgba(80.0 / 255.0, 1.0, 120.0 / 255.0, a).unwrap());
    p
}

fn sprite(out: &mut Pixmap, s: &Sprite, x: f32, y: f32, scale: f32) {
    let mut px = Pixmap::new(s.width, s.height).unwrap();
    px.data_mut().copy_from_slice(&s.rgba);
    let at = ((x + s.left) * scale, (y + s.top) * scale);
    out.draw_pixmap(at.0.round() as i32, at.1.round() as i32, px.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
}

fn text(out: &mut Pixmap, s: &str, x: f32, y: f32, scale: f32) {
    let (mut cov, t) = canvas(-MARGIN, -MARGIN, W + 2.0 * MARGIN, H + 2.0 * MARGIN, scale);
    halo::text(&mut cov, s, x, y, scale, t);
    let mask = Mask::from_pixmap(cov.as_ref(), tiny_skia::MaskType::Alpha);
    let full = tiny_skia::Rect::from_xywh(0.0, 0.0, out.width() as f32, out.height() as f32).unwrap();
    out.fill_rect(full, &paint(1.0), Transform::identity(), Some(&mask));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scale: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(2.0);
    let (mut out, t) = canvas(-MARGIN, -MARGIN, W + 2.0 * MARGIN, H + 2.0 * MARGIN, scale);
    let (m, line) = (MARGIN, Stroke { width: 1.0, ..Default::default() });
    let border = rounded(0.5, 0.5, W - 1.0, H - 1.0, 4.0);
    sprite(&mut out, &outer(scale), m, m, scale);
    out.stroke_path(&border, &paint(0.35), &line, t, None);
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba(3.0 / 255.0, 10.0 / 255.0, 3.0 / 255.0, 0.25).unwrap());
    out.fill_path(&border, &bg, FillRule::Winding, t, None);
    let mut clip = Mask::new(out.width(), out.height()).unwrap();
    clip.fill_path(&border, FillRule::Winding, true, t);
    let mut grid = PathBuilder::new();
    for y in (0..H as i32).step_by(12) {
        grid.move_to(0.0, y as f32 + 0.5);
        grid.line_to(W, y as f32 + 0.5);
    }
    for x in (0..W as i32).step_by(12) {
        grid.move_to(x as f32 + 0.5, 0.0);
        grid.line_to(x as f32 + 0.5, H);
    }
    out.stroke_path(&grid.finish().unwrap(), &paint(0.06), &line, t, Some(&clip));
    sprite(&mut out, &inner(scale), m, m, scale);
    out.stroke_path(&border, &paint(0.35), &line, t, None);
    sprite(&mut out, &title(scale), m, m, scale);
    text(&mut out, "MOTORS", TITLE.0, TITLE.1, scale);
    for i in 0..4 {
        let ring = PathBuilder::from_circle(pie_x(i), PIE_Y, R).unwrap();
        out.stroke_path(&ring, &paint(0.5), &line, t, None);
    }
    let digits = digits(scale);
    for i in 0..4 {
        let f = pwm(2.0, i);
        let (cx, a) = (pie_x(i), -std::f32::consts::FRAC_PI_2 + f * std::f32::consts::TAU);
        let mut w = PathBuilder::new();
        w.move_to(cx, PIE_Y);
        let steps = 128;
        for k in 0..=steps {
            let b = -std::f32::consts::FRAC_PI_2 + (a + std::f32::consts::FRAC_PI_2) * k as f32 / steps as f32;
            w.line_to(cx + R * b.cos(), PIE_Y + R * b.sin());
        }
        w.close();
        out.fill_path(&w.finish().unwrap(), &paint(0.6), FillRule::Winding, t, None);
        let n = format!("{:.0}", 1000.0 + f * 1000.0);
        let left = centered(cx, &n);
        for (k, d) in n.chars().enumerate() {
            sprite(&mut out, &digits[d.to_digit(10).unwrap() as usize], m + left + k as f32 * ADVANCE, m + NUMBER_Y, scale);
        }
        text(&mut out, &n, left, NUMBER_Y, scale);
    }
    out.save_png(args.get(2).map_or("preview.png", |s| s.as_str())).unwrap();
}
