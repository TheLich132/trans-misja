use std::{
    env,
    fs::File,
    io::Write,
    path::Path,
    cell::Cell,
    rc::Rc,
    cell::RefCell,
};
use crate::wav::compute_signal;
use glib_macros::clone;
use gtk4::{
    prelude::*,
    gdk,
    glib
};
use reqwest::blocking::get;

const UNET_MODEL_URL: &str = "https://huggingface.co/TempUser123/NOAA_U-Net/resolve/main/model.onnx?download=true";

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

    let debug: bool = env::var("DEBUG").map_or(false, |v| v == "1");
    let benchmark_ram: bool = env::var("BENCH_RAM").map_or(false, |v| v == "1");
    let benchmark_cpu: bool = env::var("BENCH_CPU").map_or(false, |v| v == "1");
    let sync = Rc::new(Cell::new(false));
    let use_model = Rc::new(Cell::new(false));

    let window = Rc::new(gtk4::ApplicationWindow::builder()
        .application(app)
        .title("trans-misja")
        .build());
    let window_clone = Rc::clone(&window);

    window.set_default_size(800, 600);

    let text_box = gtk4::Entry::new();
    text_box.set_placeholder_text(Some("Select a WAV file..."));

    let button_proceed = gtk4::Button::with_label("Proceed");
    button_proceed.set_sensitive(false);

    let button_open_file = gtk4::Button::with_label("Open File");
    button_open_file.connect_clicked(clone!(#[strong] text_box, #[weak] button_proceed, move |_| {
        let file_dialog = gtk4::FileDialog::new();
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("WAV files"));
        filter.add_mime_type("audio/x-wav");
        let filter_store = gio::ListStore::with_type(gtk4::FileFilter::static_type());
        filter_store.append(&filter);
        file_dialog.set_filters(Some(&filter_store));

        file_dialog.open(Some(&gtk4::Window::default()), None::<&gio::Cancellable>, clone!(#[strong] text_box, #[weak] button_proceed, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    text_box.set_text(&path.to_string_lossy());
                    button_proceed.set_sensitive(true);
                }
            }
        }));
    }));

    let checkbox_sync = gtk4::CheckButton::with_label("Sync");
    checkbox_sync.set_active(false);

    let sync_clone = Rc::clone(&sync);
    checkbox_sync.connect_toggled(move |checkbox_sync| {
        println!("Sync: {}", checkbox_sync.is_active());
        sync_clone.set(checkbox_sync.is_active());
    });

    let checkbox_use_model = Rc::new(gtk4::CheckButton::with_label("Enhance image (U-Net)"));
    checkbox_use_model.set_active(false);
    
    let use_model_clone = Rc::clone(&use_model);
    let checkbox_use_model_clone = Rc::new(RefCell::new(Rc::clone(&checkbox_use_model)));
    checkbox_use_model.connect_toggled(clone!(#[strong] use_model_clone, #[strong] window, move |checkbox| {
        println!("Enhance image: {}", checkbox.is_active());
        use_model_clone.set(checkbox.is_active());

        if checkbox.is_active() {
            let model_path = Path::new("model.onnx");
            if !model_path.exists() {
                let dialog = gtk4::AlertDialog::builder()
                    .message("The U-Net model file is missing. Would you like to download it?")
                    .buttons(["Yes", "No"])
                    .modal(true)
                    .build();

                let answer = dialog.choose_future(Some(window.as_ref()));

                let checkbox_use_model_clone = Rc::clone(&checkbox_use_model_clone);
                let progress_window = gtk4::Window::builder()
                    .title("Downloading Model")
                    .default_width(300)
                    .default_height(100)
                    .modal(true)
                    .transient_for(window.as_ref())
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

                glib::MainContext::default().spawn_local(async move {
                    let checkbox_use_model_clone = checkbox_use_model_clone.borrow();
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
                                                let progress = downloaded as f64 / total_size as f64;
                                                progress_bar.set_fraction(progress);
                                                progress_bar.set_text(Some(&format!(
                                                    "Downloading... {:.0}%",
                                                    progress * 100.0
                                                )));
                                            } else {
                                                eprintln!("Failed to write the model to file.");
                                            }
                                            if downloaded == total_size {
                                                println!("Model downloaded successfully.");
                                                checkbox_use_model_clone.set_active(true);
                                            }
                                        }
                                        Err(e) => eprintln!("Failed to create the model file: {}", e),
                                    }
                                }
                                Ok(response) => eprintln!("Failed to download the model. HTTP Status: {}", response.status()),
                                Err(e) => eprintln!("Failed to send the request to download the model: {}", e),
                            }
                        }
                        Ok(1) => checkbox_use_model_clone.set_active(false),
                        Err(e) => eprintln!("Error occurred while awaiting dialog response: {}", e),
                        _ => {}
                    }
                    progress_window.close();
                });
            }
        }
    }));
    
    let main_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    main_vbox.set_hexpand(true);
    main_vbox.set_vexpand(true);

    let top_grid = gtk4::Grid::new();
    top_grid.set_column_spacing(12);
    top_grid.set_row_spacing(12);
    top_grid.set_margin_top(12);
    top_grid.set_margin_bottom(12);
    top_grid.set_margin_start(12);
    top_grid.set_margin_end(12);

    top_grid.attach(&text_box, 0, 0, 2, 1);
    text_box.set_hexpand(true);

    top_grid.attach(&button_open_file, 2, 0, 1, 1);
    button_open_file.set_hexpand(false);

    let checkbox_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    checkbox_box.append(&checkbox_sync);
    checkbox_box.append(&*checkbox_use_model);

    top_grid.attach(&button_proceed, 2, 1, 1, 1);
    top_grid.attach(&checkbox_box, 0, 1, 2, 1);

    main_vbox.append(&top_grid);

    // Create an picture widget
    let picture_widget = gtk4::Picture::new(); 
    picture_widget.set_hexpand(true);
    picture_widget.set_vexpand(true);

    // Add the picture widget to the main vbox
    main_vbox.append(&picture_widget);

    // Create a progress bar
    let progress_bar = gtk4::ProgressBar::new();
    progress_bar.set_margin_bottom(12);
    progress_bar.set_margin_start(12);
    progress_bar.set_margin_end(12);
    progress_bar.set_hexpand(true);
    progress_bar.set_vexpand(false);
    progress_bar.set_show_text(true);

    // Add the progress bar to the main vbox
    main_vbox.append(&progress_bar);

    // Po kliknięciu przycisku "Proceed" wywołaj compute_signal z globals
    let sync_clone = Rc::clone(&sync);
    let use_model_clone = Rc::clone(&use_model);
    button_proceed.connect_clicked(clone!(#[weak] text_box, #[weak] picture_widget, #[strong] debug, #[strong] benchmark_ram, #[strong] benchmark_cpu, #[weak] sync_clone, #[weak] use_model_clone, #[weak] progress_bar, move |_| {
        let filename = text_box.text();
        if !filename.is_empty() {
            let path = compute_signal(&filename, &debug, &benchmark_ram, &benchmark_cpu, &sync_clone.get(), &use_model_clone.get(), &progress_bar);
            if !path.is_empty() {
                let file = gio::File::for_path(&path);
                picture_widget.set_file(Some(&file));
            }
        }
    }));
    window_clone.set_child(Some(&main_vbox));
    let window = Rc::clone(&window);
    window.present();
}
