use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub debug: bool,
    pub benchmark_ram: bool,
    pub benchmark_cpu: bool,
    pub sync: AtomicBool,
    pub use_model: AtomicBool,
    pub use_sgbnr: AtomicBool,
    // You can add more shared state as needed: e.g., ProgressBar, etc.
}

impl AppState {
    pub fn new(debug: bool, benchmark_ram: bool, benchmark_cpu: bool) -> Self {
        Self {
            debug,
            benchmark_ram,
            benchmark_cpu,
            sync: AtomicBool::new(false),
            use_model: AtomicBool::new(false),
            use_sgbnr: AtomicBool::new(false),
        }
    }
}
