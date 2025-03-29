use std::cell::Cell;

pub struct AppState {
    pub debug: bool,
    pub benchmark_ram: bool,
    pub benchmark_cpu: bool,
    pub sync: Cell<bool>,
    pub use_model: Cell<bool>,
    // You can add more shared state as needed: e.g., ProgressBar, etc.
}

impl AppState {
    pub fn new(debug: bool, benchmark_ram: bool, benchmark_cpu: bool) -> Self {
        Self {
            debug,
            benchmark_ram,
            benchmark_cpu,
            sync: Cell::new(false),
            use_model: Cell::new(false),
        }
    }
}