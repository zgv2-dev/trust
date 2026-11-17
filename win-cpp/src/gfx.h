// C++ port of win/ (win-d2d): Win32 + DirectComposition + Direct2D/DirectWrite.
#pragma once
#include <windows.h>
#include <d3d11.h>
#include <dxgi1_3.h>
#include <d2d1_1.h>
#include <d2d1_1helper.h>
#include <d2d1effects.h>
#include <dcomp.h>
#include <dwrite_3.h>
#include <wrl/client.h>
#include <vector>

using Microsoft::WRL::ComPtr;

// Prints the failing call and exits: the bench reports the variant as "exited early".
void check(HRESULT hr, const char* what);
#define HR(x) check((x), #x)

constexpr float W = 240.0f, H = 110.0f; // panel size in DIPs
constexpr float MARGIN = 8.0f;           // room around the panel for its outer glow

// One D3D11/D2D/DirectComposition device for the process.
struct Gpu {
    ComPtr<IDXGIDevice> dxgi;
    ComPtr<IDXGIFactory2> swapFactory;
    ComPtr<ID2D1Factory1> factory;
    ComPtr<ID2D1DeviceContext> dc;
    ComPtr<IDCompositionDevice> dcomp;
    ComPtr<IDWriteTextFormat> title, value;
    float baseline; // distance from a text layout's top to its baseline: DrawText places by box
};

// One flip-model swap chain per window, shown through a DirectComposition visual.
struct Surface {
    ComPtr<IDXGISwapChain1> swap;
    ComPtr<ID2D1Bitmap1> target;
    ComPtr<ID2D1Bitmap1> stat;  // static parts with their glows, baked once (bake)
    ComPtr<ID2D1Bitmap1> layer; // the numbers, drawn here first so they can be glowed
    ComPtr<ID2D1Effect> glow;   // the numbers' glow: shadow of `layer`
    D2D1_SIZE_U size;
    float dpi;
    ComPtr<IDCompositionTarget> compTarget;
    ComPtr<IDCompositionVisual> visual;
};

struct Brushes {
    ComPtr<ID2D1SolidColorBrush> bg, grid, border, band, ring, wedge, text;
};

Gpu makeGpu(float dpi);
Surface makeSurface(const Gpu& g, HWND hwnd, UINT width, UINT height, float dpi);
Brushes makeBrushes(ID2D1DeviceContext* dc);
ComPtr<ID2D1Bitmap1> makeBitmap(const Gpu& g, D2D1_SIZE_U size, float dpi);
// Canvas-style shadow of `input`: its alpha blurred (sigma = shadowBlur / 2), tinted accent.
ComPtr<ID2D1Effect> makeShadow(const Gpu& g, ID2D1Bitmap1* input, float sigma, float alpha);

// Panel, grid, glows, borders, title and rings of every panel into `s.stat`, once.
void bake(const Gpu& g, const Brushes& b, const Surface& s, const std::vector<D2D1_POINT_2F>& origins);

// One frame of every panel (top-left corners in `origins`, in DIPs) onto `s`; presents nothing.
void drawFrame(const Gpu& g, const Brushes& b, const Surface& s, const std::vector<D2D1_POINT_2F>& origins, float t);
