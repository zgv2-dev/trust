// C++ / Win32 variant: windows without redirection surfaces, drawn with Direct2D into
// DirectComposition swap chains. BENCH_WINDOWS widgets, one window each, or all in one overlay
// window over the primary monitor when BENCH_MODE=overlay. Redraws once per DWM frame.
#include "gfx.h"
#include <dwmapi.h>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>

static LRESULT CALLBACK wndproc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_DESTROY) {
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static D2D1_POINT_2F origin(int i) { return D2D1::Point2F(40.0f + 260.0f * (i % 4), 60.0f + 140.0f * (i / 4)); }

static int px(float v, float scale) { return (int)lroundf(v * scale); }

// Clicks outside the panels fall through to the app below (same as the clip crate).
static void clipToPanels(HWND hwnd, int n, float scale) {
    HRGN region = CreateRectRgn(0, 0, 0, 0);
    for (int i = 0; i < n; i++) {
        D2D1_POINT_2F o = origin(i);
        HRGN rect = CreateRectRgn(px(o.x, scale), px(o.y, scale), px(o.x + W, scale), px(o.y + H, scale));
        CombineRgn(region, region, rect, RGN_OR);
        DeleteObject(rect);
    }
    SetWindowRgn(hwnd, region, TRUE); // the system owns the region from here
}

struct Window {
    int x, y, w, h; // physical pixels
    std::vector<D2D1_POINT_2F> origins; // panel origins inside it, in DIPs
};

extern "C" int bench_main() {
    const char* count = getenv("BENCH_WINDOWS");
    int n = count ? atoi(count) : 4;
    const char* mode = getenv("BENCH_MODE");
    bool overlay = mode && strcmp(mode, "overlay") == 0;
    if (!SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)) {
        check(HRESULT_FROM_WIN32(GetLastError()), "SetProcessDpiAwarenessContext");
    }
    float dpi = (float)GetDpiForSystem();
    float scale = dpi / 96.0f;
    HINSTANCE instance = GetModuleHandleW(nullptr);
    WNDCLASSW cls = {};
    cls.lpfnWndProc = wndproc;
    cls.hInstance = instance;
    cls.lpszClassName = L"bench-win-cpp";
    cls.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    RegisterClassW(&cls);
    Gpu gpu = makeGpu(dpi);
    Brushes brushes = makeBrushes(gpu.dc.Get());

    std::vector<Window> layout;
    if (overlay) {
        Window w = {0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN), {}};
        for (int i = 0; i < n; i++) w.origins.push_back(origin(i));
        layout.push_back(w);
    } else {
        for (int i = 0; i < n; i++) {
            D2D1_POINT_2F o = origin(i);
            // the panel plus MARGIN on every side for its outer glow
            layout.push_back({px(o.x - MARGIN, scale), px(o.y - MARGIN, scale), px(W + 2 * MARGIN, scale),
                              px(H + 2 * MARGIN, scale), {D2D1::Point2F(MARGIN, MARGIN)}});
        }
    }
    std::vector<std::pair<Surface, std::vector<D2D1_POINT_2F>>> windows;
    for (auto& w : layout) {
        HWND hwnd = CreateWindowExW(WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                                    L"bench-win-cpp", L"bench-win-cpp", WS_POPUP | WS_VISIBLE, w.x, w.y, w.w, w.h, nullptr, nullptr, instance, nullptr);
        if (!hwnd) check(HRESULT_FROM_WIN32(GetLastError()), "CreateWindowExW");
        if (overlay) clipToPanels(hwnd, n, scale);
        windows.emplace_back(makeSurface(gpu, hwnd, w.w, w.h, dpi), w.origins);
        bake(gpu, brushes, windows.back().first, w.origins);
    }

    // BENCH_FREEZE: draw that instant forever, for screenshots
    const char* freezeEnv = getenv("BENCH_FREEZE");
    float freeze = freezeEnv ? (float)atof(freezeEnv) : -1.0f;
    bool first = true;
    using clock = std::chrono::steady_clock;
    auto start = clock::now(), last = start;
    int frames = 0;
    MSG msg;
    for (;;) {
        while (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_QUIT) return 0;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        float t = freeze >= 0 ? freeze : std::chrono::duration<float>(clock::now() - start).count();
        for (auto& w : windows) {
            drawFrame(gpu, brushes, w.first, w.second, t);
            HR(w.first.swap->Present(0, 0));
        }
        // one wait per frame for all windows; Present(1) per swap chain would stack N vsync waits
        HR(DwmFlush());
        if (first) {
            printf("first-frame\n");
            fflush(stdout);
            first = false;
        }
        frames++;
        float elapsed = std::chrono::duration<float>(clock::now() - last).count();
        if (elapsed >= 1.0f) {
            printf("fps %.0f\n", frames / elapsed);
            fflush(stdout); // stdout is a pipe to the bench, so it is fully buffered
            frames = 0;
            last = clock::now();
        }
    }
}
