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

static ComPtr<IDWriteTextFormat> format(IDWriteFactory* dwrite, IDWriteFontCollection* fonts, float size) {
    ComPtr<IDWriteTextFormat> f;
    HR(dwrite->CreateTextFormat(L"Space Mono", fonts, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STYLE_NORMAL,
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
    // Space Mono straight from fonts/, through a private font collection (no install needed)
    ComPtr<IDWriteFactory5> dwrite;
    HR(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, __uuidof(IDWriteFactory5),
                           reinterpret_cast<IUnknown**>(dwrite.GetAddressOf())));
    ComPtr<IDWriteFontFile> file;
    HR(dwrite->CreateFontFileReference(FONT_PATH, nullptr, &file));
    ComPtr<IDWriteFontSetBuilder1> builder;
    HR(dwrite->CreateFontSetBuilder(&builder));
    HR(builder->AddFontFile(file.Get()));
    ComPtr<IDWriteFontSet> set;
    HR(builder->CreateFontSet(&set));
    ComPtr<IDWriteFontCollection1> fonts;
    HR(dwrite->CreateFontCollectionFromFontSet(set.Get(), &fonts));
    g.title = format(dwrite.Get(), fonts.Get(), 12.0f);
    g.value = format(dwrite.Get(), fonts.Get(), 12.0f);
    HR(g.value->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER));
    ComPtr<IDWriteTextLayout> layout;
    HR(dwrite->CreateTextLayout(L"0", 1, g.title.Get(), 100, 100, &layout));
    DWRITE_LINE_METRICS line;
    UINT32 lines = 0;
    HR(layout->GetLineMetrics(&line, 1, &lines));
    g.baseline = line.baseline;
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
    s.size = D2D1::SizeU(width, height);
    s.dpi = dpi;
    s.layer = makeBitmap(g, s.size, dpi);
    s.stat = makeBitmap(g, s.size, dpi);
    s.glow = makeShadow(g, s.layer.Get(), 4.0f, 0.6f); // canvas shadowBlur 8 = standard deviation 4
    HR(g.dcomp->CreateTargetForHwnd(hwnd, TRUE, &s.compTarget));
    HR(g.dcomp->CreateVisual(&s.visual));
    HR(s.visual->SetContent(s.swap.Get()));
    HR(s.compTarget->SetRoot(s.visual.Get()));
    HR(g.dcomp->Commit());
    return s;
}

ComPtr<ID2D1Bitmap1> makeBitmap(const Gpu& g, D2D1_SIZE_U size, float dpi) {
    ComPtr<ID2D1Bitmap1> bmp;
    auto p = props(D2D1_BITMAP_OPTIONS_TARGET, dpi);
    HR(g.dc->CreateBitmap(size, nullptr, 0, &p, &bmp));
    return bmp;
}

ComPtr<ID2D1Effect> makeShadow(const Gpu& g, ID2D1Bitmap1* input, float sigma, float alpha) {
    ComPtr<ID2D1Effect> e;
    HR(g.dc->CreateEffect(CLSID_D2D1Shadow, &e));
    e->SetInput(0, input);
    HR(e->SetValue(D2D1_SHADOW_PROP_COLOR, D2D1::Vector4F(80 / 255.0f, 1.0f, 120 / 255.0f, alpha)));
    HR(e->SetValue(D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION, sigma));
    return e;
}
