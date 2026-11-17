// GPU plumbing. DirectComposition is what lets a swap chain keep per-pixel alpha on the desktop.
#include "gfx.h"
#include <cstdio>

void check(HRESULT hr, const char* what) {
    if (FAILED(hr)) {
        fprintf(stderr, "%s failed: 0x%08lx\n", what, (unsigned long)hr);
        ExitProcess(1);
    }
}

static D2D1_BITMAP_PROPERTIES1 props(D2D1_BITMAP_OPTIONS options, float dpi) {
    return D2D1::BitmapProperties1(
        options, D2D1::PixelFormat(DXGI_FORMAT_B8G8R8A8_UNORM, D2D1_ALPHA_MODE_PREMULTIPLIED), dpi, dpi);
}

static ComPtr<IDWriteTextFormat> format(IDWriteFactory* dwrite, float size) {
    ComPtr<IDWriteTextFormat> f;
    HR(dwrite->CreateTextFormat(L"Consolas", nullptr, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STYLE_NORMAL,
                                DWRITE_FONT_STRETCH_NORMAL, size, L"en-us", &f));
    return f;
}

Gpu makeGpu(float dpi) {
    Gpu g;
    ComPtr<ID3D11Device> d3d;
    // BGRA support is required for Direct2D on D3D11
    HR(D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, D3D11_CREATE_DEVICE_BGRA_SUPPORT, nullptr, 0,
                         D3D11_SDK_VERSION, &d3d, nullptr, nullptr));
    HR(d3d.As(&g.dxgi));
    HR(CreateDXGIFactory2(0, IID_PPV_ARGS(&g.swapFactory)));
    HR(D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, g.factory.GetAddressOf()));
    ComPtr<ID2D1Device> device;
    HR(g.factory->CreateDevice(g.dxgi.Get(), &device));
    HR(device->CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &g.dc));
    g.dc->SetDpi(dpi, dpi);
    HR(DCompositionCreateDevice(g.dxgi.Get(), IID_PPV_ARGS(&g.dcomp)));
    ComPtr<IDWriteFactory> dwrite;
    HR(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, __uuidof(IDWriteFactory),
                           reinterpret_cast<IUnknown**>(dwrite.GetAddressOf())));
    g.title = format(dwrite.Get(), 12.0f);
    g.value = format(dwrite.Get(), 12.0f);
    HR(g.value->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER));
    return g;
}

Surface makeSurface(const Gpu& g, HWND hwnd, UINT width, UINT height, float dpi) {
    Surface s;
    DXGI_SWAP_CHAIN_DESC1 desc = {};
    desc.Width = width;
    desc.Height = height;
    desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
    desc.SampleDesc.Count = 1;
    desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    desc.BufferCount = 2;
    desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;
    desc.AlphaMode = DXGI_ALPHA_MODE_PREMULTIPLIED;
    HR(g.swapFactory->CreateSwapChainForComposition(g.dxgi.Get(), &desc, nullptr, &s.swap));
    ComPtr<IDXGISurface> back;
    HR(s.swap->GetBuffer(0, IID_PPV_ARGS(&back)));
    auto targetProps = props(D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW, dpi);
    HR(g.dc->CreateBitmapFromDxgiSurface(back.Get(), &targetProps, &s.target));
    auto layerProps = props(D2D1_BITMAP_OPTIONS_TARGET, dpi);
    HR(g.dc->CreateBitmap(D2D1::SizeU(width, height), nullptr, 0, &layerProps, &s.layer));
    HR(g.dc->CreateEffect(CLSID_D2D1Shadow, &s.glow));
    s.glow->SetInput(0, s.layer.Get());
    HR(s.glow->SetValue(D2D1_SHADOW_PROP_COLOR, D2D1::Vector4F(80 / 255.0f, 1.0f, 120 / 255.0f, 0.8f)));
    HR(s.glow->SetValue(D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION, 3.0f));
    HR(g.dcomp->CreateTargetForHwnd(hwnd, TRUE, &s.compTarget));
    HR(g.dcomp->CreateVisual(&s.visual));
    HR(s.visual->SetContent(s.swap.Get()));
    HR(s.compTarget->SetRoot(s.visual.Get()));
    HR(g.dcomp->Commit());
    return s;
}
