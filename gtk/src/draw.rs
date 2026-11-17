// Draws the Motors panel with Cairo (CPU raster); GTK uploads it and composites on the GPU.
use gtk::cairo::{Context, Error, FontSlant, FontWeight};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

pub const W: f64 = 240.0;
pub const H: f64 = 110.0;
const R: f64 = 20.0;

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

/// Draws one panel with its top-left corner at (x, y), in logical pixels.
pub fn panel(cr: &Context, x: f64, y: f64, t: f64) -> Result<(), Error> {
    cr.set_line_width(1.0);
    rounded(cr, x + 0.5, y + 0.5, W - 1.0, H - 1.0, 4.0);
    cr.set_source_rgba(3.0 / 255.0, 10.0 / 255.0, 3.0 / 255.0, 0.25);
    cr.fill()?;
    accent(cr, 0.06);
    for gy in (0..H as i32).step_by(12) {
        cr.move_to(x, y + gy as f64 + 0.5);
        cr.line_to(x + W, y + gy as f64 + 0.5);
    }
    for gx in (0..W as i32).step_by(12) {
        cr.move_to(x + gx as f64 + 0.5, y);
        cr.line_to(x + gx as f64 + 0.5, y + H);
    }
    cr.stroke()?;
    rounded(cr, x + 0.5, y + 0.5, W - 1.0, H - 1.0, 4.0);
    accent(cr, 0.35);
    cr.stroke()?;

    cr.select_font_face("monospace", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(12.0);
    accent(cr, 1.0);
    cr.move_to(x + 10.0, y + 20.0);
    cr.show_text("MOTORS")?;
    for i in 0..4 {
        let f = 0.5 + 0.45 * (t * 1.3 + i as f64).sin(); // fake PWM
        let (cx, cy) = (x + 32.0 + 52.0 * i as f64, y + 50.0);
        accent(cr, 0.5);
        cr.new_sub_path();
        cr.arc(cx, cy, R, 0.0, TAU);
        cr.stroke()?;
        accent(cr, 0.6);
        cr.move_to(cx, cy);
        cr.arc(cx, cy, R, -FRAC_PI_2, -FRAC_PI_2 + f * TAU);
        cr.close_path();
        cr.fill()?;
        let text = format!("{:.0}", 1000.0 + f * 1000.0);
        let w = cr.text_extents(&text)?.width();
        accent(cr, 1.0);
        cr.move_to(cx - w / 2.0, y + 94.0);
        cr.show_text(&text)?;
    }
    Ok(())
}
