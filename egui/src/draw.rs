// Draws the Motors panel the way the reference does (tauri/dist/widget.js), step by step and in
// the same order: halo's pre-blurred images for the shadows egui can't draw, the rest with egui.
use eframe::egui::{self, pos2, vec2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, TextureHandle};
use halo::{ADVANCE, H, NUMBER_Y, R, TITLE, W};

/// A glow uploaded as a texture, and where it goes relative to its anchor, in points.
struct Glow(TextureHandle, Rect);

pub struct Glows {
    outer: Glow,
    inner: Glow,
    title: Glow,
    digits: Vec<Glow>,
    pub scale: f32,
}

fn upload(ctx: &egui::Context, name: &str, s: &halo::Sprite, scale: f32) -> Glow {
    let image = egui::ColorImage::from_rgba_premultiplied([s.width as usize, s.height as usize], &s.rgba);
    let tex = ctx.load_texture(name, image, egui::TextureOptions::LINEAR);
    let size = vec2(s.width as f32 / scale, s.height as f32 / scale);
    Glow(tex, Rect::from_min_size(pos2(s.left, s.top), size))
}

impl Glows {
    pub fn new(ctx: &egui::Context, scale: f32) -> Self {
        let digits = halo::digits(scale).into_iter().enumerate().map(|(d, s)| upload(ctx, &format!("digit{d}"), &s, scale));
        Glows {
            outer: upload(ctx, "outer", &halo::outer(scale), scale),
            inner: upload(ctx, "inner", &halo::inner(scale), scale),
            title: upload(ctx, "title", &halo::title(scale), scale),
            digits: digits.collect(),
            scale,
        }
    }
}

fn accent(a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(80, 255, 120, (a * 255.0).round() as u8)
}

fn image(p: &Painter, g: &Glow, at: Pos2) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    p.image(g.0.id(), g.1.translate(at.to_vec2()), uv, Color32::WHITE);
}

/// `s` with its baseline starting at `at`: egui places text by its top, so shift by the baseline.
fn text(p: &Painter, s: &str, at: Pos2, color: Color32) {
    let galley = p.layout_no_wrap(s.to_owned(), FontId::monospace(halo::FONT_SIZE), color);
    let row = &galley.rows[0];
    let baseline = row.pos.y + row.row.glyphs.first().map_or(halo::ASCENT, |g| g.pos.y);
    p.galley(pos2(at.x, at.y - baseline), galley, color);
}

/// Pie wedge from 12 o'clock clockwise; egui fills only convex shapes, so halves of at most 180°.
fn wedge(p: &Painter, c: Pos2, f: f32) {
    let start = -std::f32::consts::FRAC_PI_2;
    let sweep = f * std::f32::consts::TAU;
    let half = sweep.min(std::f32::consts::PI);
    for (a0, a1) in [(start, start + half), (start + half, start + sweep)] {
        if a1 - a0 <= 0.0 {
            continue;
        }
        let steps = ((a1 - a0) / 0.05).ceil().max(1.0) as usize;
        let mut pts = vec![c];
        pts.extend((0..=steps).map(|k| c + R * egui::Vec2::angled(a0 + (a1 - a0) * k as f32 / steps as f32)));
        p.add(Shape::convex_polygon(pts, accent(0.6), Stroke::NONE));
    }
}

/// One panel with its top-left corner at `o`, at time `t`.
pub fn panel(p: &Painter, o: Pos2, t: f32, g: &Glows) {
    let border = Rect::from_min_size(o + vec2(0.5, 0.5), vec2(W - 1.0, H - 1.0));
    let stroke = Stroke::new(1.0, accent(0.35));
    image(p, &g.outer, o); // outer glow, then the border it belongs to
    p.rect_stroke(border, 4.0, stroke, StrokeKind::Middle);
    p.rect_filled(border, 4.0, Color32::from_rgba_unmultiplied(3, 10, 3, 64));
    let grid = p.with_clip_rect(Rect::from_min_size(o, vec2(W, H)).intersect(p.clip_rect()));
    for y in (0..H as i32).step_by(12) {
        let y = o.y + y as f32 + 0.5;
        grid.line_segment([pos2(o.x, y), pos2(o.x + W, y)], Stroke::new(1.0, accent(0.06)));
    }
    for x in (0..W as i32).step_by(12) {
        let x = o.x + x as f32 + 0.5;
        grid.line_segment([pos2(x, o.y), pos2(x, o.y + H)], Stroke::new(1.0, accent(0.06)));
    }
    image(p, &g.inner, o);
    p.rect_stroke(border, 4.0, stroke, StrokeKind::Middle);
    image(p, &g.title, o);
    text(p, "MOTORS", o + vec2(TITLE.0, TITLE.1), accent(1.0));
    for i in 0..4 {
        p.circle_stroke(o + vec2(halo::pie_x(i), halo::PIE_Y), R, Stroke::new(1.0, accent(0.5)));
    }
    for i in 0..4 {
        let f = halo::pwm(t, i);
        let c = o + vec2(halo::pie_x(i), halo::PIE_Y);
        wedge(p, c, f);
        let s = format!("{:.0}", 1000.0 + f * 1000.0);
        let x = halo::centered(halo::pie_x(i), &s);
        for (k, d) in s.chars().enumerate() {
            let d = d.to_digit(10).unwrap_or(0) as usize;
            image(p, &g.digits[d], o + vec2(x + k as f32 * ADVANCE, NUMBER_Y));
        }
        text(p, &s, o + vec2(x, NUMBER_Y), accent(1.0));
    }
}
