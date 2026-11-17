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

// One D3D11/D2D/DirectComposition device for the process.
struct Gpu {
    ComPtr<IDXGIDevice> dxgi;
    ComPtr<IDXGIFactory2> swapFactory;
    ComPtr<ID2D1Factory1> factory;
    ComPtr<ID2D1DeviceContext> dc;
    ComPtr<IDCompositionDevice> dcomp;
    ComPtr<IDWriteTextFormat> title, value;
};

// One flip-model swap chain per window, shown through a DirectComposition visual.
struct Surface {
    ComPtr<IDXGISwapChain1> swap;
    ComPtr<ID2D1Bitmap1> target;
    ComPtr<ID2D1Bitmap1> layer; // foreground drawn here first, so it can be glowed and composited
    ComPtr<ID2D1Effect> glow;
    ComPtr<IDCompositionTarget> compTarget;
    ComPtr<IDCompositionVisual> visual;
};

struct Brushes {
    ComPtr<ID2D1SolidColorBrush> bg, grid, border, ring, wedge, text;
};

Gpu makeGpu(float dpi);
Surface makeSurface(const Gpu& g, HWND hwnd, UINT width, UINT height, float dpi);
Brushes makeBrushes(ID2D1DeviceContext* dc);

// Draws every panel (top-left corners in `origins`, in DIPs) onto `s` and presents nothing.
void drawFrame(const Gpu& g, const Brushes& b, const Surface& s, const std::vector<D2D1_POINT_2F>& origins, float t);
