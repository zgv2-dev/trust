// Draws the Motors panel the way the reference does (tauri/dist/widget.js), in the same order.
// The static parts and their glows (GPU shadow effects with the reference's blur) are baked once
// per window; each frame blits them and draws the wedges and the numbers with their glow.
use crate::gfx::{Gpu, Surface};
use std::f32::consts::{FRAC_PI_2, TAU};
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::{IDWriteTextFormat, DWRITE_MEASURING_MODE_NATURAL};
use windows_numerics::Vector2;

pub const W: f32 = 240.0;
pub const H: f32 = 110.0;
/// Room around the panel for its outer glow: a window of its own is this much larger per side.
pub const MARGIN: f32 = 8.0;
const R: f32 = 20.0;

pub struct Brushes {
    bg: ID2D1SolidColorBrush,
    grid: ID2D1SolidColorBrush,
    border: ID2D1SolidColorBrush,
    band: ID2D1SolidColorBrush,
    ring: ID2D1SolidColorBrush,
    wedge: ID2D1SolidColorBrush,
    text: ID2D1SolidColorBrush,
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a }
}

impl Brushes {
    pub unsafe fn new(dc: &ID2D1DeviceContext) -> Result<Self> {
        let b = |c: D2D1_COLOR_F| dc.CreateSolidColorBrush(&c, None);
        Ok(Self {
            bg: b(rgba(3, 10, 3, 0.25))?,
            grid: b(rgba(80, 255, 120, 0.06))?,
            border: b(rgba(80, 255, 120, 0.35))?,
            band: b(rgba(80, 255, 120, 0.15))?,
            ring: b(rgba(80, 255, 120, 0.5))?,
            wedge: b(rgba(80, 255, 120, 0.6))?,
            text: b(rgba(80, 255, 120, 1.0))?,
        })
    }
}

fn pt(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}

fn pie_x(i: usize) -> f32 {
    32.0 + 52.0 * i as f32
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> D2D1_ROUNDED_RECT {
    D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: x, top: y, right: x + w, bottom: y + h }, radiusX: r, radiusY: r }
}

/// Text with its baseline at `y`, in a box from `left` to `right` (aligned by the format).
unsafe fn text(g: &Gpu, b: &Brushes, s: &str, f: &IDWriteTextFormat, left: f32, right: f32, y: f32) {
    let s: Vec<u16> = s.encode_utf16().collect();
    let rect = D2D_RECT_F { left, top: y - g.baseline, right, bottom: y - g.baseline + 30.0 };
    g.dc.DrawText(&s, f, &rect, &b.text, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
}

/// A bitmap the size of the window holding what `draw` paints, as input for a shadow.
unsafe fn mask(g: &Gpu, s: &Surface, draw: impl Fn(&ID2D1DeviceContext)) -> Result<ID2D1Bitmap1> {
    let bmp = g.bitmap(s.size, s.dpi)?;
    g.dc.SetTarget(&bmp);
    g.dc.BeginDraw();
    g.dc.Clear(Some(&D2D1_COLOR_F::default()));
    draw(&g.dc);
    g.dc.EndDraw(None, None)?;
    Ok(bmp)
}

unsafe fn image(dc: &ID2D1DeviceContext, effect: &ID2D1Effect) -> Result<()> {
    let out: ID2D1Image = effect.GetOutput()?;
    dc.DrawImage(&out, None, None, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    Ok(())
}

/// Panel, grid, glows, borders, title and rings of every panel (top-left corners in `origins`)
/// into `s.stat`, once.
pub unsafe fn bake(g: &Gpu, b: &Brushes, s: &Surface, origins: &[(f32, f32)]) -> Result<()> {
    let border = |dc: &ID2D1DeviceContext| {
        for &(x, y) in origins {
            dc.DrawRoundedRectangle(&rounded(x + 0.5, y + 0.5, W - 1.0, H - 1.0, 4.0), &b.border, 1.0, None);
        }
    };
    let border_mask = mask(g, s, border)?;
    let band_mask = mask(g, s, |dc| {
        for &(x, y) in origins {
            dc.DrawRoundedRectangle(&rounded(x - 1.0, y - 1.0, W + 2.0, H + 2.0, 4.0), &b.band, 2.0, None);
        }
    })?;
    let title_mask = mask(g, s, |_| origins.iter().for_each(|&(x, y)| text(g, b, "MOTORS", &g.title, x + 10.0, x + W, y + 20.0)))?;
    // shadowBlur 6 / 24 / 6 = standard deviation 3 / 12 / 3
    let (outer, inner, title) = (g.shadow(&border_mask, 3.0, 0.45)?, g.shadow(&band_mask, 12.0, 0.5)?, g.shadow(&title_mask, 3.0, 1.0)?);
    let dc = &g.dc;
    dc.SetTarget(&s.stat);
    dc.BeginDraw();
    dc.Clear(Some(&D2D1_COLOR_F::default()));
    image(dc, &outer)?;
    border(dc);
    for &(x, y) in origins {
        dc.FillRoundedRectangle(&rounded(x + 0.5, y + 0.5, W - 1.0, H - 1.0, 4.0), &b.bg);
        let clip = D2D_RECT_F { left: x + 0.5, top: y + 0.5, right: x + W - 0.5, bottom: y + H - 0.5 };
        dc.PushAxisAlignedClip(&clip, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        for gy in (0..H as i32).step_by(12) {
            dc.DrawLine(pt(x, y + gy as f32 + 0.5), pt(x + W, y + gy as f32 + 0.5), &b.grid, 1.0, None);
        }
        for gx in (0..W as i32).step_by(12) {
            dc.DrawLine(pt(x + gx as f32 + 0.5, y), pt(x + gx as f32 + 0.5, y + H), &b.grid, 1.0, None);
        }
        image(dc, &inner)?; // the inner glow, clipped to this panel
        dc.PopAxisAlignedClip();
    }
    border(dc);
    image(dc, &title)?;
    for &(x, y) in origins {
        text(g, b, "MOTORS", &g.title, x + 10.0, x + W, y + 20.0);
        for i in 0..4 {
            let ring = D2D1_ELLIPSE { point: pt(x + pie_x(i), y + 50.0), radiusX: R, radiusY: R };
            dc.DrawEllipse(&ring, &b.ring, 1.0, None);
        }
    }
    dc.EndDraw(None, None)
}

/// One frame: the baked static parts, then wedges and numbers (with their glow) at time `t`.
pub unsafe fn frame(g: &Gpu, b: &Brushes, s: &Surface, origins: &[(f32, f32)], t: f32) -> Result<()> {
    let dc = &g.dc;
    let value = |i: usize| 0.5 + 0.45 * (t * 1.3 + i as f32).sin(); // fake PWM
    dc.SetTarget(&s.layer);
    dc.BeginDraw();
    dc.Clear(Some(&D2D1_COLOR_F::default()));
    for &(x, y) in origins {
        for i in 0..4 {
            let n = format!("{:.0}", 1000.0 + value(i) * 1000.0);
            text(g, b, &n, &g.value, x + pie_x(i) - 26.0, x + pie_x(i) + 26.0, y + 94.0);
        }
    }
    dc.EndDraw(None, None)?;

    dc.SetTarget(&s.target);
    dc.BeginDraw();
    dc.Clear(Some(&D2D1_COLOR_F::default()));
    dc.DrawImage(&s.stat, None, None, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    for &(x, y) in origins {
        for i in 0..4 {
            dc.FillGeometry(&wedge(&g.factory, pt(x + pie_x(i), y + 50.0), value(i))?, &b.wedge, None);
        }
    }
    image(dc, &s.glow)?;
    dc.DrawImage(&s.layer, None, None, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    dc.EndDraw(None, None)
}

unsafe fn wedge(factory: &ID2D1Factory1, c: Vector2, f: f32) -> Result<ID2D1PathGeometry1> {
    let path = factory.CreatePathGeometry()?;
    let sink = path.Open()?;
    let a = -FRAC_PI_2 + f * TAU;
    sink.BeginFigure(c, D2D1_FIGURE_BEGIN_FILLED);
    sink.AddLine(pt(c.X, c.Y - R));
    sink.AddArc(&D2D1_ARC_SEGMENT {
        point: pt(c.X + R * a.cos(), c.Y + R * a.sin()),
        size: D2D_SIZE_F { width: R, height: R },
        rotationAngle: 0.0,
        sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
        arcSize: if f > 0.5 { D2D1_ARC_SIZE_LARGE } else { D2D1_ARC_SIZE_SMALL },
    });
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;
    Ok(path)
}
