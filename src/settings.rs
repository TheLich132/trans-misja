use crate::settings_logic::connect_settings_logic;
use crate::ui_elements::UiElements;

use std::sync::Arc;
use std::sync::Mutex;

pub struct FunctionsSettings {
    //Low pass filter settings
    pub cutoff_freq: f32,
    // Sync apt settings
    pub additional_offset: usize,
    // Envelope detection settings
    pub window_size: usize,
    pub scaling_factor: f32,
    // Enhance image settings
    pub cpu_threads: usize,
    // SGBNR settings
    pub blur_sigma: f32,
    pub brightness_threshold: f32,
    pub noise_threshold: f32,
    pub sharpen_sigma: f32,
    pub sharpen_threshold: i32,
}

impl FunctionsSettings {
    pub fn new(ui_elements: &UiElements) -> Arc<Mutex<Self>> {
        // Create instance with default values
        let settings = Arc::new(Mutex::new(Self {
            cutoff_freq: 5000.0,
            additional_offset: 120,
            window_size: 10,
            scaling_factor: 2.5,
            cpu_threads: 1,
            blur_sigma: 8.0,
            brightness_threshold: 5.0,
            noise_threshold: 27.5,
            sharpen_sigma: 1.5,
            sharpen_threshold: 5,
        }));
        // Connect UI elements to settings
        connect_settings_logic(ui_elements, &settings);

        settings
    }

    pub fn new_without_ui() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            cutoff_freq: 5000.0,
            additional_offset: 120,
            window_size: 10,
            scaling_factor: 2.5,
            cpu_threads: 1,
            blur_sigma: 8.0,
            brightness_threshold: 5.0,
            noise_threshold: 27.5,
            sharpen_sigma: 1.5,
            sharpen_threshold: 5,
        }))
    }
}
