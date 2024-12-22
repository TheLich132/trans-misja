use std::env;
use crate::wav::compute_signal;
use gtk4::prelude::*;
use gtk4::{gdk, glib};
use std::cell::Cell;
use std::rc::Rc;

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
    let sync = Rc::new(Cell::new(false));
    let use_model = Rc::new(Cell::new(false));

    let text_box = gtk4::Entry::new();
    text_box.set_placeholder_text(Some("Select a WAV file..."));

    let button_proceed = gtk4::Button::with_label("Proceed");
    button_proceed.set_sensitive(false);

    let button_open_file = gtk4::Button::with_label("Open File");
    button_open_file.connect_clicked(glib::clone!(@weak app, @weak text_box, @weak button_proceed => move |_| {
        let dialog = gtk4::FileChooserDialog::new(
            Some("Select a WAV file"),
            Some(&app.active_window().unwrap()),
            gtk4::FileChooserAction::Open,
            &[("Open", gtk4::ResponseType::Ok), ("Cancel", gtk4::ResponseType::Cancel)],
        );
        dialog.set_select_multiple(false);
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("WAV files"));
        filter.add_mime_type("audio/x-wav");
        dialog.add_filter(&filter);
        dialog.connect_response(glib::clone!(@weak app, @weak text_box, @weak button_proceed => move |dialog, response| {
            if response == gtk4::ResponseType::Ok {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        text_box.set_text(path.to_str().unwrap());
                        button_proceed.set_sensitive(true); // Upewnij się, że przycisk jest włączony po wyborze pliku
                    }
                }
            }
            dialog.close();
        }));
        dialog.show();
    }));

    let checkbox_sync = gtk4::CheckButton::with_label("Sync");
    checkbox_sync.set_active(false);

    let sync_clone = Rc::clone(&sync);
    checkbox_sync.connect_toggled(move |checkbox_sync| {
        println!("Sync: {}", checkbox_sync.is_active());
        sync_clone.set(checkbox_sync.is_active());
    });

    let checkbox_use_model = gtk4::CheckButton::with_label("Enhance image");
    checkbox_use_model.set_active(false);

    let use_model_clone = Rc::clone(&use_model);
    checkbox_use_model.connect_toggled(move |checkbox_use_model| {
        println!("Enhance image: {}", checkbox_use_model.is_active());
        use_model_clone.set(checkbox_use_model.is_active());
    });
    
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
    checkbox_box.append(&checkbox_use_model);

    top_grid.attach(&button_proceed, 2, 1, 1, 1);
    top_grid.attach(&checkbox_box, 0, 1, 2, 1);

    main_vbox.append(&top_grid);

    // Create an picture widget
    let picture_widget = gtk4::Picture::new(); 
    picture_widget.set_hexpand(true);
    picture_widget.set_vexpand(true);

    // Add the picture widget to the main vbox
    main_vbox.append(&picture_widget);

    // Po kliknięciu przycisku "Proceed" wywołaj compute_signal z globals
    let sync_clone = Rc::clone(&sync);
    let use_model_clone = Rc::clone(&use_model);
    button_proceed.connect_clicked(glib::clone!(@weak text_box, @weak picture_widget, @strong debug, @weak sync_clone, @weak use_model_clone => move |_| {
        let filename = text_box.text();
        if !filename.is_empty() {
            let path = compute_signal(&filename, &debug, &sync_clone.get(), &use_model_clone.get());
            if !path.is_empty() {
                picture_widget.set_file(Some());
            }
        }
    }));

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("trans-misja")
        .child(&main_vbox)
        .build();

    window.set_default_size(800, 600);
    window.present();
}
