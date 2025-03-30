use crate::app_state::AppState;
use crate::settings::FunctionsSettings;
use crate::ui_elements::UiElements;
use crate::wav::compute_signal;

use glib_macros::clone;
use gtk4::{gdk, glib, prelude::*};
use reqwest::blocking::get;
use std::{env, fs::File, io::Write, path::Path, rc::Rc};

const UNET_MODEL_URL: &str =
    "https://huggingface.co/TempUser123/NOAA_U-Net/resolve/main/model.onnx?download=true";

fn load_css() {
    let settings = gio::Settings::new("org.gnome.desktop.interface");
    let color_scheme = settings.string("color-scheme");
    let theme = settings.string("gtk-theme");
    let is_dark_theme = color_scheme.eq("prefer-dark");

    let theme_path = if !is_dark_theme {
        format!("/usr/share/themes/{}/gtk-4.0/gtk.css", theme)
    } else {
        format!(
            "/usr/share/themes/{}/gtk-4.0/gtk.css",
            theme.to_string() + "-dark"
        )
    };

    let provider = gtk4::CssProvider::new();
    provider.load_from_path(theme_path);

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn build_ui(app: &gtk4::Application) {
    load_css();

    let debug: bool = env::var("DEBUG").is_ok_and(|v| v == "1");
    let benchmark_ram: bool = env::var("BENCH_RAM").is_ok_and(|v| v == "1");
    let benchmark_cpu: bool = env::var("BENCH_CPU").is_ok_and(|v| v == "1");

    // Initialize object to hold shared state
    // Use Rc to allow multiple ownership of the AppState object
    let app_state = Rc::new(AppState::new(debug, benchmark_ram, benchmark_cpu));
    //Initialize object to hold UI elements
    let ui_elements = Rc::new(UiElements::new(app));
    // Initialize object to hold settings
    let settings = FunctionsSettings::new(&ui_elements);

    // Logic for filepicker
    ui_elements.button_open_file.connect_clicked(clone!(
        #[strong]
        ui_elements,
        move |_| {
            let file_dialog = gtk4::FileDialog::new();
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("WAV files"));
            filter.add_mime_type("audio/x-wav");
            let filter_store = gio::ListStore::with_type(gtk4::FileFilter::static_type());
            filter_store.append(&filter);
            file_dialog.set_filters(Some(&filter_store));
            file_dialog.set_modal(true);

            file_dialog.open(
                Some(&ui_elements.window),
                None::<&gio::Cancellable>,
                clone!(
                    #[strong]
                    ui_elements,
                    move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                ui_elements.text_box.set_text(&path.to_string_lossy());
                                ui_elements.button_proceed.set_sensitive(true);
                            }
                        }
                    }
                ),
            );
        }
    ));

    // Logic for settings button
    ui_elements.button_settings.connect_clicked(clone!(
        #[strong]
        ui_elements,
        move |_| {
            ui_elements.present_settings();
        }
    ));

    // Logic for proceed button
    ui_elements.checkbox_sync.connect_toggled(clone!(
        #[strong]
        app_state,
        move |checkbox_sync| {
            println!("Sync: {}", checkbox_sync.is_active());
            app_state.sync.set(checkbox_sync.is_active());
        }
    ));

    // Logic for use model checkbox
    ui_elements.checkbox_use_model.connect_toggled(clone!(
        #[strong]
        app_state,
        #[strong]
        ui_elements,
        move |checkbox| {
            println!("Enhance image: {}", checkbox.is_active());
            app_state.use_model.set(checkbox.is_active());

            if checkbox.is_active() {
                let model_path = Path::new("model.onnx");
                if !model_path.exists() {
                    let dialog = gtk4::AlertDialog::builder()
                        .message("The U-Net model file is missing. Would you like to download it?")
                        .buttons(["Yes", "No"])
                        .modal(true)
                        .build();

                    let answer = dialog.choose_future(Some(&ui_elements.window));

                    let progress_window = gtk4::Window::builder()
                        .title("Downloading Model")
                        .default_width(300)
                        .default_height(100)
                        .modal(true)
                        .transient_for(&ui_elements.window)
                        .build();
                    progress_window.set_resizable(false);

                    let progress_bar = gtk4::ProgressBar::new();
                    progress_bar.set_show_text(true);
                    progress_bar.set_text(Some("Starting download..."));
                    progress_bar.set_margin_top(12);
                    progress_bar.set_margin_bottom(12);
                    progress_bar.set_margin_start(12);
                    progress_bar.set_margin_end(12);

                    progress_window.set_child(Some(&progress_bar));
                    progress_window.present();

                    let ui_elements_clone = ui_elements.clone();
                    glib::MainContext::default().spawn_local(async move {
                        match answer.await {
                            Ok(0) => {
                                println!("Downloading the model...");
                                match get(UNET_MODEL_URL) {
                                    Ok(response) if response.status().is_success() => {
                                        let total_size = response.content_length().unwrap_or(0);
                                        match File::create("model.onnx") {
                                            Ok(mut file) => {
                                                let mut downloaded: u64 = 0;
                                                let content = match response.bytes() {
                                                    Ok(bytes) => bytes,
                                                    Err(_) => {
                                                        eprintln!("Failed to read response bytes.");
                                                        return;
                                                    }
                                                };
                                                if file.write_all(&content).is_ok() {
                                                    downloaded += content.len() as u64;
                                                    let progress =
                                                        downloaded as f64 / total_size as f64;
                                                    progress_bar.set_fraction(progress);
                                                    progress_bar.set_text(Some(&format!(
                                                        "Downloading... {:.0}%",
                                                        progress * 100.0
                                                    )));
                                                } else {
                                                    eprintln!("Failed to write the model to file.");
                                                    if downloaded == total_size {
                                                        println!("Model downloaded successfully.");
                                                        ui_elements_clone
                                                            .checkbox_sync
                                                            .set_active(true);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to create the model file: {}", e)
                                            }
                                        }
                                    }
                                    Ok(response) => eprintln!(
                                        "Failed to download the model. HTTP Status: {}",
                                        response.status()
                                    ),
                                    Err(e) => eprintln!(
                                        "Failed to send the request to download the model: {}",
                                        e
                                    ),
                                }
                            }
                            Ok(1) => {
                                ui_elements_clone.checkbox_use_model.set_active(false);
                            }
                            Err(e) => {
                                eprintln!("Error occurred while awaiting dialog response: {}", e)
                            }
                            _ => {}
                        }
                        progress_window.close();
                    });
                }
            }
        }
    ));

    // Logic for the proceed button
    ui_elements.button_proceed.connect_clicked(clone!(
        #[strong]
        ui_elements,
        #[strong]
        app_state,
        #[strong]
        settings,
        move |_| {
            let filename = &ui_elements.text_box.text();
            if !filename.is_empty() {
                let path = compute_signal(filename, &app_state, &ui_elements, &settings.borrow());
                if !path.is_empty() {
                    let file = gio::File::for_path(&path);
                    ui_elements.picture_widget.set_file(Some(&file));
                }
            }
        }
    ));

    ui_elements.window.present();
}
