use crate::settings::FunctionsSettings;
use crate::ui_elements::UiElements;

use glib_macros::clone;
use std::sync::{Arc, Mutex};

pub fn connect_settings_logic(ui_elements: &UiElements, settings: &Arc<Mutex<FunctionsSettings>>) {
    // Cutoff frequency settings
    ui_elements
        .cutoff_frequency_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.cutoff_freq = spin_button.value() as f32;
                    println!("Cutoff frequency set to: {}", s.cutoff_freq);
                }
            }
        ));

    // Additional offset settings
    ui_elements
        .additional_offset_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.additional_offset = spin_button.value() as usize;
                    println!("Additional offset set to: {}", s.additional_offset);
                }
            }
        ));

    // Window size settings
    ui_elements
        .window_size_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.window_size = spin_button.value() as usize;
                    println!("Window size set to: {}", s.window_size);
                }
            }
        ));

    // Scaling factor settings
    ui_elements
        .scaling_factor_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.scaling_factor = spin_button.value() as f32;
                    println!("Scaling factor set to: {}", s.scaling_factor);
                }
            }
        ));

    // CPU threads settings
    ui_elements
        .cpu_threads_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.cpu_threads = spin_button.value() as usize;
                    println!("CPU threads set to: {}", s.cpu_threads);
                }
            }
        ));

    // Blur sigma settings
    ui_elements
        .blur_sigma_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.blur_sigma = spin_button.value() as f32;
                    println!("Blur sigma set to: {}", s.blur_sigma);
                }
            }
        ));

    // Brightness threshold settings
    ui_elements
        .brightness_threshold_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.brightness_threshold = spin_button.value() as f32;
                    println!("Brightness threshold set to: {}", s.brightness_threshold);
                }
            }
        ));

    // Noise threshold settings
    ui_elements
        .noise_threshold_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.noise_threshold = spin_button.value() as f32;
                    println!("Noise threshold set to: {}", s.noise_threshold);
                }
            }
        ));

    // Sharpen sigma settings
    ui_elements
        .sharpen_sigma_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.sharpen_sigma = spin_button.value() as f32;
                    println!("Sharpen sigma set to: {}", s.sharpen_sigma);
                }
            }
        ));

    // Sharpen threshold settings
    ui_elements
        .sharpen_threshold_spinbutton
        .connect_value_changed(clone!(
            #[strong]
            settings,
            move |spin_button| {
                if let Ok(mut s) = settings.lock() {
                    s.sharpen_threshold = spin_button.value() as i32;
                    println!("Sharpen threshold set to: {}", s.sharpen_threshold);
                }
            }
        ));
}
