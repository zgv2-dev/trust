// Compiles the C++ variant with MSVC (cc finds it, no Developer Prompt needed) and links the
// Windows libraries it uses. Off Windows nothing is built and main.rs only prints an error.
fn main() {
    println!("cargo:rerun-if-changed=src");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    cc::Build::new()
        .cpp(true)
        .files(["src/main.cpp", "src/gfx.cpp", "src/draw.cpp"])
        .define("UNICODE", None)
        .define("NOMINMAX", None)
        .compile("wincpp");
    for lib in ["d3d11", "dxgi", "d2d1", "dcomp", "dwrite", "dwmapi", "dxguid", "user32", "gdi32"] {
        println!("cargo:rustc-link-lib={lib}");
    }
}
