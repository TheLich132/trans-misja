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
    let globals: GLOBALS = GLOBALS {
        debug
    };

    // Text box with path
    let text_box = gtk4::Entry::new();
    text_box.set_placeholder_text(Some("Select a WAV file..."));

    // Create button "Procede"
    let button_procede: gtk4::Button = gtk4::Button::with_label("Procede");
    button_procede.set_sensitive(false);

    // Create a button
    let button = gtk4::Button::with_label("Open File");
    button.connect_clicked(glib::clone!(@weak app, @weak text_box, @weak button_procede => move |_| {
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
                        button_procede.set_sensitive(true);
                    }
                }
            }
            dialog.close();
        }));
        dialog.show();
    }));

    let main_grid_box = gtk4::Grid::new();
    main_grid_box.set_row_spacing(12);
    main_grid_box.set_column_spacing(12);
    main_grid_box.set_column_homogeneous(false);

    // Create a GridBox
    let grid_box = gtk4::Grid::new();
    grid_box.set_margin_top(12);
    grid_box.set_margin_bottom(12);
    grid_box.set_margin_start(12);
    grid_box.set_margin_end(12);
    grid_box.set_row_spacing(12);
    grid_box.set_column_spacing(12);
    grid_box.set_column_homogeneous(true);

    // ********************************
    // TODO: Move button_procede to the bottom of the window
    // ********************************
    
    // Add widgets to the GridBox
    grid_box.attach(&text_box, 0, 0, 2, 1);
    grid_box.attach(&button, 2, 0, 1, 1);
    grid_box.attach(&button_procede, 0, 1, 3, 1);

    // Limit the size of the GridBox
    grid_box.set_halign(gtk4::Align::Start);
    grid_box.set_valign(gtk4::Align::Start);
    grid_box.set_hexpand(false);
    grid_box.set_vexpand(false);

    main_grid_box.attach(&grid_box, 0, 0, 1, 1);

    // Connect the button to the load_wav_file function
    button_procede.connect_clicked(glib::clone!(@weak app, @weak text_box, @weak main_grid_box => move |_| {
        let mut path: String = String::new();
        let filename = text_box.text();
        if !filename.is_empty() {
            path = compute_signal(filename.as_str(), &globals);
        }
        if!path.is_empty() {
            // ********************************
            // TODO: Fix image expansion to the whole available space
            // ********************************
            
            // Create a new image and add it to the main grid box
            let img = gtk4::Image::new();
            img.set_from_file(Some(path));
            img.set_halign(gtk4::Align::Fill);
            img.set_valign(gtk4::Align::Fill);
            img.set_hexpand(true);
            img.set_vexpand(true);
            
            let img_grid_box = gtk4::Grid::new();
            img_grid_box.set_column_homogeneous(true);
            img_grid_box.set_row_homogeneous(true);
            img_grid_box.set_halign(gtk4::Align::Fill);
            img_grid_box.set_valign(gtk4::Align::Fill);
            img_grid_box.set_hexpand(true);
            img_grid_box.set_vexpand(true);

            img_grid_box.attach(&img, 0, 0, 1, 1);
            main_grid_box.attach(&img_grid_box, 1, 0, 1, 1);

            main_grid_box.show();
        }
    }));

    // Create a window
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("trans-misja")
        .child(&main_grid_box)
        .build();

    // Present the window
    window.present();
}
