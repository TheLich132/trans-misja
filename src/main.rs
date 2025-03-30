use gtk4::glib;
use gtk4::prelude::*;

mod app_state;
mod gaussian_blur;
mod settings;
mod settings_logic;
mod ui;
mod ui_elements;
mod wav;

const APP_ID: &str = "org.gtk-rs.trans-misja";

fn main() -> glib::ExitCode {
    let app = gtk4::Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run()
}
