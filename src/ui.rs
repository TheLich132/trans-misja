use crate::wav::load_wav_file;
use gtk4::prelude::*;
use gtk4::{gdk, glib};

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

    // Connect the button to the load_wav_file function
    button_procede.connect_clicked(glib::clone!(@weak app, @weak text_box => move |_| {
        let filename = text_box.text();
        if !filename.is_empty() {
            load_wav_file(filename.as_str());
        }
    }));

    // Create a GridBox
    let grid_box = gtk4::Grid::new();
    grid_box.set_margin_top(12);
    grid_box.set_margin_bottom(12);
    grid_box.set_margin_start(12);
    grid_box.set_margin_end(12);
    grid_box.set_row_spacing(12);
    grid_box.set_column_spacing(12);
    grid_box.set_column_homogeneous(true);

    // Add widgets to the GridBox
    grid_box.attach(&text_box, 0, 0, 2, 1);
    grid_box.attach(&button, 2, 0, 1, 1);
    grid_box.attach(&button_procede, 0, 1, 3, 1);


    // Create a window
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("trans-misja")
        .child(&grid_box)
        .build();

    window.set_default_size(500, 100);

    // Present the window
    window.present();
}
