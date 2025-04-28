use gtk4::glib;
use gtk4::prelude::*;
use std::env;

mod app_state;
mod console_command;
mod gaussian_blur;
mod settings;
mod settings_logic;
mod ui_elements;
mod ui_logic;
mod wav;

const APP_ID: &str = "org.gtk-rs.trans-misja";

fn main() -> glib::ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let function_settings = settings::FunctionsSettings::new_without_ui();
        let img_path = &args[1];

        console_command::generate_images(img_path, function_settings);

        return glib::ExitCode::SUCCESS;
    }

    let app = gtk4::Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui_logic::build_ui);
    app.run()
}
