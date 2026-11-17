// Draws the Motors panel with Cairo the way the reference does (tauri/dist/widget.js), step by step
// and in the same order; halo's pre-blurred images stand in for the shadows Cairo can't draw.
use gtk::cairo::{Context, Error, FontSlant, FontWeight, Format, ImageSurface};
use halo::{ADVANCE, H, NUMBER_Y, R, TITLE, W};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// A glow as a Cairo surface (device scale = screen scale) and its offset in logical px.
struct Glow(ImageSurface, f64, f64);

pub struct Glows {
    outer: Glow,
    inner: Glow,
    title: Glow,
    digits: Vec<Glow>,
    pub scale: i32,
}

fn surface(s: &halo::Sprite, scale: f64) -> Glow {
    // Cairo's ARGB32 is premultiplied, native-endian: B, G, R, A in memory on little-endian
    let mut data: Vec<u8> = s.rgba.chunks(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect();
    let stride = Format::ARgb32.stride_for_width(s.width).unwrap();
    if stride as u32 != s.width * 4 {
        let rows = data.chunks(s.width as usize * 4);
        data = rows.flat_map(|r| r.iter().copied().chain(std::iter::repeat(0).take(stride as usize - r.len()))).collect();
    }
    let img = ImageSurface::create_for_data(data, Format::ARgb32, s.width as i32, s.height as i32, stride).unwrap();
    img.set_device_scale(scale, scale);
    Glow(img, s.left as f64, s.top as f64)
}

impl Glows {
    pub fn new(scale: i32) -> Self {
        let f = scale as f32;
        let s = |sprite: halo::Sprite| surface(&sprite, scale as f64);
        Glows {
            outer: s(halo::outer(f)),
            inner: s(halo::inner(f)),
            title: s(halo::title(f)),
            digits: halo::digits(f).into_iter().map(s).collect(),
            scale,
        }
    }
}

fn accent(cr: &Context, a: f64) {
    cr.set_source_rgba(80.0 / 255.0, 1.0, 120.0 / 255.0, a);
}

fn rounded(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    cr.arc(x + r, y + r, r, PI, 1.5 * PI);
    cr.close_path();
}

fn image(cr: &Context, g: &Glow, x: f64, y: f64) -> Result<(), Error> {
    cr.set_source_surface(&g.0, x + g.1, y + g.2)?;
    cr.paint()
}

/// Draws one panel with its top-left corner at (x, y), in logical pixels.
pub fn panel(cr: &Context, x: f64, y: f64, t: f64, g: &Glows) -> Result<(), Error> {
    let (w, h) = (W as f64, H as f64);
    cr.set_line_width(1.0);
    image(cr, &g.outer, x, y)?; // outer glow, then the border it belongs to
    rounded(cr, x + 0.5, y + 0.5, w - 1.0, h - 1.0, 4.0);
    accent(cr, 0.35);
    cr.stroke()?;
    rounded(cr, x + 0.5, y + 0.5, w - 1.0, h - 1.0, 4.0);
    cr.set_source_rgba(3.0 / 255.0, 10.0 / 255.0, 3.0 / 255.0, 0.25);
    cr.fill_preserve()?;
    cr.save()?;
    cr.clip();
    accent(cr, 0.06);
    for gy in (0..H as i32).step_by(12) {
        cr.move_to(x, y + gy as f64 + 0.5);
        cr.line_to(x + w, y + gy as f64 + 0.5);
    }
    for gx in (0..W as i32).step_by(12) {
        cr.move_to(x + gx as f64 + 0.5, y);
        cr.line_to(x + gx as f64 + 0.5, y + h);
    }
    cr.stroke()?;
    cr.restore()?;
    image(cr, &g.inner, x, y)?;
    rounded(cr, x + 0.5, y + 0.5, w - 1.0, h - 1.0, 4.0);
    accent(cr, 0.35);
    cr.stroke()?;

    cr.select_font_face("Space Mono", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(halo::FONT_SIZE as f64);
    image(cr, &g.title, x, y)?;
    accent(cr, 1.0);
    cr.move_to(x + TITLE.0 as f64, y + TITLE.1 as f64);
    cr.show_text("MOTORS")?;
    accent(cr, 0.5);
    for i in 0..4 {
        cr.new_sub_path();
        cr.arc(x + halo::pie_x(i) as f64, y + halo::PIE_Y as f64, R as f64, 0.0, TAU);
    }
    cr.stroke()?;
    for i in 0..4 {
        let f = halo::pwm(t as f32, i) as f64;
        let (cx, cy) = (x + halo::pie_x(i) as f64, y + halo::PIE_Y as f64);
        accent(cr, 0.6);
        cr.move_to(cx, cy);
        cr.arc(cx, cy, R as f64, -FRAC_PI_2, -FRAC_PI_2 + f * TAU);
        cr.close_path();
        cr.fill()?;
        let text = format!("{:.0}", 1000.0 + f * 1000.0);
        let left = halo::centered(halo::pie_x(i), &text) as f64;
        for (k, d) in text.chars().enumerate() {
            let d = d.to_digit(10).unwrap_or(0) as usize;
            image(cr, &g.digits[d], x + left + k as f64 * ADVANCE as f64, y + NUMBER_Y as f64)?;
        }
        accent(cr, 1.0);
        cr.move_to(x + left, y + NUMBER_Y as f64);
        cr.show_text(&text)?;
    }
    Ok(())
}
