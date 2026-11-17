// Entry point only: everything the variant does is the C++ in this folder (bench_main in main.cpp).
#[cfg(windows)]
extern "C" {
    fn bench_main() -> i32;
}

fn main() {
    #[cfg(windows)]
    std::process::exit(unsafe { bench_main() });
    #[cfg(not(windows))]
    {
        eprintln!("bench-win-cpp only runs on Windows");
        std::process::exit(1);
    }
}
