use crate::settings::FunctionsSettings;
use crate::ui_elements::UiElements;

use glib_macros::clone;
use std::cell::RefCell;
use std::rc::Rc;

pub fn connect_settings_logic(ui_elements: &UiElements, settings: &Rc<RefCell<FunctionsSettings>>) {
    // Cutoff frequency settings
    ui_elements
        .cutoff_frequency_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                settings.borrow_mut().cutoff_freq = spin_button.value() as f32;
                println!("Cutoff frequency set to: {}", settings.borrow().cutoff_freq);
            }
        ));

    // Additional offset settings
    ui_elements
        .additional_offset_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                settings.borrow_mut().additional_offset = spin_button.value() as usize;
                println!(
                    "Additional offset set to: {}",
                    settings.borrow().additional_offset
                );
            }
        ));

    // Window size settings
    ui_elements
        .window_size_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                settings.borrow_mut().window_size = spin_button.value() as usize;
                println!("Window size set to: {}", settings.borrow().window_size);
            }
        ));

    // Scaling factor settings
    ui_elements
        .scaling_factor_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                settings.borrow_mut().scaling_factor = spin_button.value() as f32;
                println!(
                    "Scaling factor set to: {}",
                    settings.borrow().scaling_factor
                );
            }
        ));

    // CPU threads settings
    ui_elements
        .cpu_threads_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                settings.borrow_mut().cpu_threads = spin_button.value() as usize;
                println!("CPU threads set to: {}", settings.borrow().cpu_threads);
            }
        ));
}
