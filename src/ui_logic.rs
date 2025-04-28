use crate::app_state::AppState;
use crate::settings::FunctionsSettings;
use crate::ui_elements::UiElements;
use crate::wav::compute_signal;

use glib_macros::clone;
use gtk4::{gdk, gio, glib, prelude::*};
use reqwest::blocking::get;
use std::sync::atomic::Ordering;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
    rc::Rc,
    sync::Arc,
};

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
    let app_state = Arc::new(AppState::new(debug, benchmark_ram, benchmark_cpu));
    //Initialize object to hold UI elements
    let ui_elements = Rc::new(UiElements::new(app));
    // Initialize object to hold settings
    let settings = FunctionsSettings::new(&ui_elements);

    let (sender, receiver) = async_channel::bounded(1);
    // Logic for filepicker
    ui_elements.button_open_file.connect_clicked(clone!(
        #[strong]
        ui_elements,
        move |_| {
            let initial_folder = ui_elements.text_box.text();
            let file_dialog = gtk4::FileDialog::new();
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("WAV files"));
            filter.add_mime_type("audio/x-wav");
            let filter_store = gio::ListStore::with_type(gtk4::FileFilter::static_type());
            filter_store.append(&filter);
            file_dialog.set_filters(Some(&filter_store));
            file_dialog.set_modal(true);
            if !initial_folder.is_empty() {
                let file = gio::File::for_path(&initial_folder);
                file_dialog.set_initial_folder(Some(&file));
            }

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
            app_state
                .sync
                .store(checkbox_sync.is_active(), Ordering::SeqCst);
        }
    ));

    // Logic for use model checkbox
    ui_elements.checkbox_use_model.connect_toggled(clone!(
        #[strong]
        app_state,
        #[strong]
        ui_elements,
        move |checkbox| {
            let sender = sender.clone();
            let is_active = checkbox.is_active();
            println!("Enhance image: {}", checkbox.is_active());
            app_state
                .use_model
                .store(checkbox.is_active(), Ordering::SeqCst);

            if is_active {
                ui_elements.checkbox_use_sgbnr.set_active(false);
            }

            if checkbox.is_active() {
                let model_path = Path::new("model.onnx");
                if !model_path.exists() {
                    let dialog = gtk4::AlertDialog::builder()
                        .message("The U-Net model file is missing. Would you like to download it?")
                        .buttons(["Yes", "No"])
                        .modal(true)
                        .build();

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

                    // clone needed values for async/blocked tasks
                    let dialog_clone = dialog.clone();
                    let sender_clone = sender.clone();
                    let ui_elements_clone = ui_elements.clone();

                    // await the user's response before proceeding
                    glib::MainContext::default().spawn_local(clone!(
                        #[strong]
                        progress_window,
                        async move {
                            if let Ok(answer) = dialog_clone
                                .choose_future(Some(&ui_elements_clone.window))
                                .await
                            {
                                if answer == 0 {
                                    // offload the blocking download to the thread pool
                                    gio::spawn_blocking(move || {
                                        let mut downloaded: u64 = 0;
                                        if let Ok(mut resp) = get(UNET_MODEL_URL) {
                                            if resp.status().is_success() {
                                                let total = resp.content_length().unwrap_or(0);
                                                if let Ok(mut file) = File::create("model.onnx") {
                                                    let mut buf = [0u8; 8192];
                                                    // read in chunks
                                                    loop {
                                                        match resp.read(&mut buf) {
                                                            Ok(0) => break,
                                                            Ok(n) => {
                                                                downloaded += n as u64;
                                                                if file.write_all(&buf[..n]).is_ok()
                                                                {
                                                                    let fraction = downloaded
                                                                        as f64
                                                                        / total as f64;
                                                                    let text = format!(
                                                                        "Downloading... {:.0}%",
                                                                        fraction * 100.0
                                                                    );
                                                                    let _ = sender_clone
                                                                        .try_send((fraction, text));
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!(
                                                                    "Error reading chunk: {}",
                                                                    e
                                                                );
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    // final update
                                                    while sender_clone
                                                        .try_send((1.0, "Download complete".into()))
                                                        .is_err()
                                                    {
                                                        std::thread::sleep(
                                                            std::time::Duration::from_millis(10),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    });
                                } else {
                                    // User chose not to download the model
                                    progress_window.close();
                                    ui_elements_clone.checkbox_use_model.set_active(false);
                                }
                            }
                        }
                    ));

                    // Update the progress bar with the download progress
                    let ui_elements_clone = ui_elements.clone();
                    let progress_window_clone = progress_window.clone();
                    let progress_rx = receiver.clone();
                    glib::MainContext::default().spawn_local(async move {
                        while let Ok((fraction, text)) = progress_rx.recv().await {
                            progress_bar.set_fraction(fraction);
                            progress_bar.set_text(Some(&text));
                            if fraction >= 1.0 {
                                progress_window_clone.close();
                                ui_elements_clone.checkbox_use_model.set_active(true);
                                break;
                            }
                        }
                    });
                }
            }
        }
    ));

    // Logic for use sgbnr checkbox
    ui_elements.checkbox_use_sgbnr.connect_toggled(clone!(
        #[strong]
        app_state,
        #[strong]
        ui_elements,
        move |checkbox| {
            let is_active = checkbox.is_active();
            println!("Use SGBNR: {}", checkbox.is_active());
            app_state
                .use_sgbnr
                .store(checkbox.is_active(), Ordering::SeqCst);

            if is_active {
                ui_elements.checkbox_use_model.set_active(false);
            }
        }
    ));

    let (sender, receiver) = async_channel::bounded(1);

    // Logic for the proceed button
    ui_elements.button_proceed.connect_clicked(clone!(
        #[strong]
        ui_elements,
        #[strong]
        app_state,
        #[strong]
        settings,
        move |_| {
            let sender = sender.clone();
            let filename = ui_elements.text_box.text().to_string();
            if !filename.is_empty() {
                gio::spawn_blocking(clone!(
                    #[strong]
                    app_state,
                    #[strong]
                    settings,
                    move || {
                        let app_state = app_state.clone();
                        let settings = settings.clone();
                        let filename = filename.to_string();
                        // Call the function to enhance the image with the model
                        compute_signal(&filename, &app_state, &settings, &sender);
                    }
                ));
            }

            // update progress bar with processing progress
            let ui_elements_clone = ui_elements.clone();
            let progress_rx = receiver.clone();
            glib::MainContext::default().spawn_local(async move {
                ui_elements_clone.button_proceed.set_sensitive(false);
                while let Ok((fraction, text)) = progress_rx.recv().await {
                    if fraction >= 1.0 {
                        ui_elements_clone
                            .progress_bar
                            .set_text(Some("Processing complete"));
                        ui_elements_clone.progress_bar.set_fraction(1.0);
                        ui_elements_clone.button_proceed.set_sensitive(true);

                        let path = text;
                        if !path.is_empty() {
                            let file = gio::File::for_path(&path);
                            ui_elements_clone.picture_widget.set_file(Some(&file));
                        }
                        break;
                    }
                    ui_elements_clone.progress_bar.set_fraction(fraction);
                    ui_elements_clone.progress_bar.set_text(Some(&text));
                }
            });
        }
    ));

    ui_elements.window.present();
}
