use std::env;
use crate::wav::compute_signal;
use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4::prelude::WidgetExt;

pub struct GLOBALS {
    pub debug: bool,
}

fn load_css() {
    // Get the color scheme from the settings
    let settings = gio::Settings::new("org.gnome.desktop.interface");
    let color_scheme = settings.string("color-scheme");
    let theme = settings.string("gtk-theme");
    let is_dark_theme = color_scheme.eq("prefer-dark");

    // Get the path to the CSS file
    let theme_path = if !is_dark_theme {
        format!("/usr/share/themes/{}/gtk-4.0/gtk.css", theme)
    } else {
        format!(
            "/usr/share/themes/{}/gtk-4.0/gtk.css",
            theme.to_string() + "-dark"
        )
    };

    // Load the CSS file and add it to the provider
    let provider = gtk4::CssProvider::new();
    provider.load_from_path(theme_path);

    // Add the provider to the default screen
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn build_ui(app: &gtk4::Application) {
    load_css();

    // Get the debug flag from the environment
    let mut debug: bool = false;
    let key: &str = "DEBUG";
    if env::var_os(key).is_some() {
        if let Ok(val) = env::var(key) {
            debug = val == "1";
        }
    }
    let globals: GLOBALS = GLOBALS { debug };

    // Text box with path
    let text_box = gtk4::Entry::new();
    text_box.set_placeholder_text(Some("Select a WAV file..."));

    // Create button "Proceed"
    let button_proceed: gtk4::Button = gtk4::Button::with_label("Proceed");
    button_proceed.set_sensitive(false);

    // Create a button
    let button_open_file = gtk4::Button::with_label("Open File");
    button_open_file.connect_clicked(glib::clone!(@weak app, @weak text_box, @weak button_proceed => move |_| {
        // Create a file chooser dialog
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
        dialog.connect_response(glib::clone!(@weak app => move |dialog, response| {
            if response == gtk4::ResponseType::Ok {
                // Open the selected file
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        text_box.set_text(path.to_str().unwrap());
                        button_proceed.set_sensitive(true);
                    }
                }
            }
            dialog.close();
        }));
        dialog.show();
    }));

    // Main vertical box to organize layout
    let main_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    // Make the window, grid, and all child widgets expand fully
    main_vbox.set_hexpand(true);
    main_vbox.set_vexpand(true);

    // Grid layout for the top part (text_box and buttons)
    let top_grid = gtk4::Grid::new();
    top_grid.set_column_spacing(12);
    top_grid.set_row_spacing(12);
    top_grid.set_margin_top(12);
    top_grid.set_margin_bottom(12);
    top_grid.set_margin_start(12);
    top_grid.set_margin_end(12);

    // Add text_box to the first column, spanning most of the width
    top_grid.attach(&text_box, 0, 0, 1, 1);
    text_box.set_hexpand(true); // Expand to full available width

    // Add button next to the text_box
    top_grid.attach(&button_open_file, 1, 0, 1, 1);
    button_open_file.set_hexpand(false); // Do not expand as much as text_box

    // Add button_proceed below button
    top_grid.attach(&button_proceed, 1, 1, 1, 1);

    // Add the top_grid to the main_vbox
    main_vbox.append(&top_grid);

    // Create a box for the image (bottom section)
    let image_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    image_box.set_hexpand(true);
    image_box.set_vexpand(true);

    // Add image_box to the main_vbox (bottom section)
    main_vbox.append(&image_box);

    // Connect the button to the compute signal function
    button_proceed.connect_clicked(glib::clone!(@weak app, @weak text_box, @weak image_box => move |_| {
        let mut path: String = String::new();
        let filename = text_box.text();
        if !filename.is_empty() {
            path = compute_signal(filename.as_str(), &globals);
        }
        if !path.is_empty() {
            // **************
            // TODO: Fix image displaying size to fit screen
            // **************

            // Create a new image and add it to the image_box
            let img = gtk4::Image::new();
            img.set_from_file(Some(path));
    
            // Make the image expand to the full available space
            img.set_hexpand(true);
            img.set_vexpand(true);
            img.set_halign(gtk4::Align::Fill);
            img.set_valign(gtk4::Align::Fill);
    
            // Make sure the image box also expands
            image_box.set_hexpand(true);
            image_box.set_vexpand(true);
            image_box.set_halign(gtk4::Align::Fill);
            image_box.set_valign(gtk4::Align::Fill);
    
            image_box.append(&img);
    
            // Show the updated layout
            image_box.show();
        }
    }));    

    // Create a window and add the main_vbox as the child
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("trans-misja")
        .child(&main_vbox)
        .build();

    window.set_default_size(800, 600); // Optional: Set a default size for the window

    window.present();
}