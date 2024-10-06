use gtk4::glib;
use gtk4::prelude::*;

mod ui;
mod wav;

const APP_ID: &str = "org.gtk-rs.trans-misja";

fn main() -> glib::ExitCode {
    let app = gtk4::Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run()
}
