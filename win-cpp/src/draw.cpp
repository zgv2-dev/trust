// Draws the Motors panel with Direct2D: backgrounds straight to the swap chain, foreground into a
// layer that gets a GPU shadow (glow) before being composited on top.
#include "gfx.h"
#include <cmath>
#include <cwchar>

static const float R = 20.0f;
static const float PI = 3.14159265358979f;

static D2D1_COLOR_F rgba(int r, int g, int b, float a) {
    return D2D1::ColorF(r / 255.0f, g / 255.0f, b / 255.0f, a);
}

Brushes makeBrushes(ID2D1DeviceContext* dc) {
    Brushes b;
    HR(dc->CreateSolidColorBrush(rgba(3, 10, 3, 0.25f), &b.bg));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.06f), &b.grid));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.35f), &b.border));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.5f), &b.ring));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.6f), &b.wedge));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 1.0f), &b.text));
    return b;
}

static float pieX(int i) { return 10.0f + 22.0f + i * 52.0f; }

static ComPtr<ID2D1PathGeometry1> wedge(ID2D1Factory1* factory, D2D1_POINT_2F c, float f) {
    ComPtr<ID2D1PathGeometry1> path;
    HR(factory->CreatePathGeometry(&path));
    ComPtr<ID2D1GeometrySink> sink;
    HR(path->Open(&sink));
    float a = -PI / 2 + f * 2 * PI;
    sink->BeginFigure(c, D2D1_FIGURE_BEGIN_FILLED);
    sink->AddLine(D2D1::Point2F(c.x, c.y - R));
    sink->AddArc(D2D1::ArcSegment(D2D1::Point2F(c.x + R * cosf(a), c.y + R * sinf(a)), D2D1::SizeF(R, R), 0.0f,
                                  D2D1_SWEEP_DIRECTION_CLOCKWISE,
                                  f > 0.5f ? D2D1_ARC_SIZE_LARGE : D2D1_ARC_SIZE_SMALL));
    sink->EndFigure(D2D1_FIGURE_END_CLOSED);
    HR(sink->Close());
    return path;
}

void drawFrame(const Gpu& g, const Brushes& b, const Surface& s, const std::vector<D2D1_POINT_2F>& origins, float t) {
    ID2D1DeviceContext* dc = g.dc.Get();
    const D2D1_COLOR_F clear = D2D1::ColorF(0, 0, 0, 0);
    dc->SetTarget(s.layer.Get());
    dc->BeginDraw();
    dc->Clear(clear);
    for (auto o : origins) {
        // the border is in the glowed layer too: that gives the panel its tiny outer glow
        D2D1_ROUNDED_RECT border = {D2D1::RectF(o.x + 0.5f, o.y + 0.5f, o.x + W - 0.5f, o.y + H - 0.5f), 4, 4};
        dc->DrawRoundedRectangle(border, b.border.Get(), 1.0f);
        D2D1_RECT_F titleRect = D2D1::RectF(o.x + 10, o.y + 6, o.x + W, o.y + 24);
        dc->DrawText(L"MOTORS", 6, g.title.Get(), &titleRect, b.text.Get());
        for (int i = 0; i < 4; i++) {
            float f = 0.5f + 0.45f * sinf(t * 1.3f + i); // fake PWM
            D2D1_POINT_2F c = D2D1::Point2F(o.x + pieX(i), o.y + 50);
            dc->DrawEllipse(D2D1::Ellipse(c, R, R), b.ring.Get(), 1.0f);
            dc->FillGeometry(wedge(g.factory.Get(), c, f).Get(), b.wedge.Get());
            wchar_t text[16];
            int len = swprintf(text, 16, L"%.0f", 1000.0f + f * 1000.0f);
            D2D1_RECT_F rect = D2D1::RectF(c.x - 26, o.y + 80, c.x + 26, o.y + 98);
            dc->DrawText(text, (UINT32)len, g.value.Get(), &rect, b.text.Get());
        }
    }
    HR(dc->EndDraw());

    dc->SetTarget(s.target.Get());
    dc->BeginDraw();
    dc->Clear(clear);
    for (auto o : origins) {
        D2D1_ROUNDED_RECT rounded = {D2D1::RectF(o.x + 0.5f, o.y + 0.5f, o.x + W - 0.5f, o.y + H - 0.5f), 4, 4};
        dc->FillRoundedRectangle(rounded, b.bg.Get());
        for (float gy = 0.5f; gy < H; gy += 12)
            dc->DrawLine(D2D1::Point2F(o.x, o.y + gy), D2D1::Point2F(o.x + W, o.y + gy), b.grid.Get(), 1.0f);
        for (float gx = 0.5f; gx < W; gx += 12)
            dc->DrawLine(D2D1::Point2F(o.x + gx, o.y), D2D1::Point2F(o.x + gx, o.y + H), b.grid.Get(), 1.0f);
    }
    ComPtr<ID2D1Image> glow;
    s.glow->GetOutput(&glow);
    dc->DrawImage(glow.Get(), D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    dc->DrawImage(s.layer.Get(), D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    HR(dc->EndDraw());
}
