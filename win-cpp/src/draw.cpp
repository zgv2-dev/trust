// Draws the Motors panel the way the reference does (tauri/dist/widget.js), in the same order.
// The static parts and their glows (GPU shadow effects with the reference's blur) are baked once
// per window; each frame blits them and draws the wedges and the numbers with their glow.
#include "gfx.h"
#include <cmath>
#include <cwchar>
#include <functional>

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
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.15f), &b.band));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.5f), &b.ring));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 0.6f), &b.wedge));
    HR(dc->CreateSolidColorBrush(rgba(80, 255, 120, 1.0f), &b.text));
    return b;
}

static float pieX(int i) { return 32.0f + i * 52.0f; }

static D2D1_ROUNDED_RECT rounded(float x, float y, float w, float h, float r) {
    return {D2D1::RectF(x, y, x + w, y + h), r, r};
}

// Text with its baseline at `y`, in a box from `left` to `right` (aligned by the format).
static void text(const Gpu& g, const Brushes& b, const wchar_t* s, IDWriteTextFormat* f, float left, float right, float y) {
    D2D1_RECT_F rect = D2D1::RectF(left, y - g.baseline, right, y - g.baseline + 30);
    g.dc->DrawText(s, (UINT32)wcslen(s), f, &rect, b.text.Get());
}

// A bitmap the size of the window holding what `draw` paints, as input for a shadow.
static ComPtr<ID2D1Bitmap1> mask(const Gpu& g, const Surface& s, const std::function<void(ID2D1DeviceContext*)>& draw) {
    ComPtr<ID2D1Bitmap1> bmp = makeBitmap(g, s.size, s.dpi);
    g.dc->SetTarget(bmp.Get());
    g.dc->BeginDraw();
    g.dc->Clear(D2D1::ColorF(0, 0, 0, 0));
    draw(g.dc.Get());
    HR(g.dc->EndDraw());
    return bmp;
}

static void image(ID2D1DeviceContext* dc, ID2D1Effect* e) {
    ComPtr<ID2D1Image> out;
    e->GetOutput(&out);
    dc->DrawImage(out.Get(), D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
}

void bake(const Gpu& g, const Brushes& b, const Surface& s, const std::vector<D2D1_POINT_2F>& origins) {
    auto border = [&](ID2D1DeviceContext* dc) {
        for (auto o : origins) dc->DrawRoundedRectangle(rounded(o.x + 0.5f, o.y + 0.5f, W - 1, H - 1, 4), b.border.Get(), 1.0f);
    };
    auto borderMask = mask(g, s, border);
    auto bandMask = mask(g, s, [&](ID2D1DeviceContext* dc) {
        for (auto o : origins) dc->DrawRoundedRectangle(rounded(o.x - 1, o.y - 1, W + 2, H + 2, 4), b.band.Get(), 2.0f);
    });
    auto titleMask = mask(g, s, [&](ID2D1DeviceContext*) {
        for (auto o : origins) text(g, b, L"MOTORS", g.title.Get(), o.x + 10, o.x + W, o.y + 20);
    });
    // shadowBlur 6 / 24 / 6 = standard deviation 3 / 12 / 3
    auto outer = makeShadow(g, borderMask.Get(), 3.0f, 0.45f);
    auto inner = makeShadow(g, bandMask.Get(), 12.0f, 0.5f);
    auto title = makeShadow(g, titleMask.Get(), 3.0f, 1.0f);
    ID2D1DeviceContext* dc = g.dc.Get();
    dc->SetTarget(s.stat.Get());
    dc->BeginDraw();
    dc->Clear(D2D1::ColorF(0, 0, 0, 0));
    image(dc, outer.Get());
    border(dc);
    for (auto o : origins) {
        dc->FillRoundedRectangle(rounded(o.x + 0.5f, o.y + 0.5f, W - 1, H - 1, 4), b.bg.Get());
        dc->PushAxisAlignedClip(D2D1::RectF(o.x + 0.5f, o.y + 0.5f, o.x + W - 0.5f, o.y + H - 0.5f), D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        for (float gy = 0.5f; gy < H; gy += 12)
            dc->DrawLine(D2D1::Point2F(o.x, o.y + gy), D2D1::Point2F(o.x + W, o.y + gy), b.grid.Get(), 1.0f);
        for (float gx = 0.5f; gx < W; gx += 12)
            dc->DrawLine(D2D1::Point2F(o.x + gx, o.y), D2D1::Point2F(o.x + gx, o.y + H), b.grid.Get(), 1.0f);
        image(dc, inner.Get()); // the inner glow, clipped to this panel
        dc->PopAxisAlignedClip();
    }
    border(dc);
    image(dc, title.Get());
    for (auto o : origins) {
        text(g, b, L"MOTORS", g.title.Get(), o.x + 10, o.x + W, o.y + 20);
        for (int i = 0; i < 4; i++) dc->DrawEllipse(D2D1::Ellipse(D2D1::Point2F(o.x + pieX(i), o.y + 50), R, R), b.ring.Get(), 1.0f);
    }
    HR(dc->EndDraw());
}

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
    auto value = [t](int i) { return 0.5f + 0.45f * sinf(t * 1.3f + i); }; // fake PWM
    dc->SetTarget(s.layer.Get());
    dc->BeginDraw();
    dc->Clear(D2D1::ColorF(0, 0, 0, 0));
    for (auto o : origins) {
        for (int i = 0; i < 4; i++) {
            wchar_t n[16];
            swprintf(n, 16, L"%.0f", 1000.0f + value(i) * 1000.0f);
            text(g, b, n, g.value.Get(), o.x + pieX(i) - 26, o.x + pieX(i) + 26, o.y + 94);
        }
    }
    HR(dc->EndDraw());

    dc->SetTarget(s.target.Get());
    dc->BeginDraw();
    dc->Clear(D2D1::ColorF(0, 0, 0, 0));
    dc->DrawImage(s.stat.Get(), D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    for (auto o : origins)
        for (int i = 0; i < 4; i++)
            dc->FillGeometry(wedge(g.factory.Get(), D2D1::Point2F(o.x + pieX(i), o.y + 50), value(i)).Get(), b.wedge.Get());
    image(dc, s.glow.Get());
    dc->DrawImage(s.layer.Get(), D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
    HR(dc->EndDraw());
}
