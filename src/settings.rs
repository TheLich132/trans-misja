use crate::settings_logic::connect_settings_logic;
use crate::ui_elements::UiElements;

use std::cell::RefCell;
use std::rc::Rc;

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
}

impl FunctionsSettings {
    pub fn new(ui_elements: &UiElements) -> Rc<RefCell<Self>> {
        // Create instance with default values
        let settings = Rc::new(RefCell::new(Self {
            cutoff_freq: 5000.0,
            additional_offset: 120,
            window_size: 20,
            scaling_factor: 1.0,
            cpu_threads: 1,
        }));
        // Connect UI elements to settings
        connect_settings_logic(ui_elements, &settings);

        settings
    }
}
