// Draws the Motors panel with Direct2D: backgrounds straight to the swap chain, foreground into a
// layer that gets a GPU shadow (glow) before being composited on top.
use crate::gfx::{Gpu, Surface};
use std::f32::consts::{FRAC_PI_2, TAU};
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL;
use windows_numerics::Vector2;

pub const W: f32 = 240.0;
pub const H: f32 = 110.0;
const R: f32 = 20.0;

pub struct Brushes {
    bg: ID2D1SolidColorBrush,
    grid: ID2D1SolidColorBrush,
    border: ID2D1SolidColorBrush,
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
    10.0 + 22.0 + i as f32 * 52.0
}

/// Draws every panel (top-left corners in `origins`, in DIPs) onto `s` and presents nothing.
pub unsafe fn frame(g: &Gpu, b: &Brushes, s: &Surface, origins: &[(f32, f32)], t: f32) -> Result<()> {
    let dc = &g.dc;
    let clear = D2D1_COLOR_F::default();
    dc.SetTarget(&s.layer);
    dc.BeginDraw();
    dc.Clear(Some(&clear));
    for &(x, y) in origins {
        let title = "MOTORS".encode_utf16().collect::<Vec<_>>();
        let rect = D2D_RECT_F { left: x + 10.0, top: y + 6.0, right: x + W, bottom: y + 24.0 };
        dc.DrawText(&title, &g.title, &rect, &b.text, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        for i in 0..4 {
            let f = 0.5 + 0.45 * (t * 1.3 + i as f32).sin(); // fake PWM
            let c = pt(x + pie_x(i), y + 50.0);
            let ring = D2D1_ELLIPSE { point: c, radiusX: R, radiusY: R };
            dc.DrawEllipse(&ring, &b.ring, 1.0, None);
            dc.FillGeometry(&wedge(&g.factory, c, f)?, &b.wedge, None);
            let text = format!("{:.0}", 1000.0 + f * 1000.0).encode_utf16().collect::<Vec<_>>();
            let rect = D2D_RECT_F { left: c.X - 26.0, top: y + 80.0, right: c.X + 26.0, bottom: y + 98.0 };
            dc.DrawText(&text, &g.value, &rect, &b.text, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
    }
    dc.EndDraw(None, None)?;

    dc.SetTarget(&s.target);
    dc.BeginDraw();
    dc.Clear(Some(&clear));
    for &(x, y) in origins {
        let rect = D2D_RECT_F { left: x + 0.5, top: y + 0.5, right: x + W - 0.5, bottom: y + H - 0.5 };
        let rounded = D2D1_ROUNDED_RECT { rect, radiusX: 4.0, radiusY: 4.0 };
        dc.FillRoundedRectangle(&rounded, &b.bg);
        let mut gy = 0.5;
        while gy < H {
            dc.DrawLine(pt(x, y + gy), pt(x + W, y + gy), &b.grid, 1.0, None);
            gy += 12.0;
        }
        let mut gx = 0.5;
        while gx < W {
            dc.DrawLine(pt(x + gx, y), pt(x + gx, y + H), &b.grid, 1.0, None);
            gx += 12.0;
        }
        dc.DrawRoundedRectangle(&rounded, &b.border, 1.0, None);
    }
    let image: ID2D1Image = s.glow.GetOutput()?;
    dc.DrawImage(&image, None, None, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
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
