use gtk4::{
    prelude::*, ApplicationWindow, Box, Button, CheckButton, Entry, HeaderBar, Label, Picture,
    ProgressBar, SpinButton, Stack, StackSwitcher, Window,
};
use sysinfo::System;

pub struct UiElements {
    pub window: ApplicationWindow,
    pub text_box: Entry,
    pub button_proceed: Button,
    pub button_open_file: Button,
    pub button_settings: Button,
    pub checkbox_sync: CheckButton,
    pub checkbox_use_model: CheckButton,
    pub picture_widget: Picture,
    pub progress_bar: ProgressBar,

    // Settings ui
    settings_window: Window,
    pub cutoff_frequency_spinbutton: SpinButton,
    pub additional_offset_spinbutton: SpinButton,
    pub window_size_spinbutton: SpinButton,
    pub scaling_factor_spinbutton: SpinButton,
    pub cpu_threads_spinbutton: SpinButton,
}

impl UiElements {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("trans-misja")
            .default_width(800)
            .default_height(600)
            .resizable(true)
            .build();

        let text_box = Entry::new();
        text_box.set_placeholder_text(Some("Select a WAV file..."));
        text_box.set_editable(false);
        text_box.set_hexpand(true);

        let button_proceed = Button::with_label("Proceed");
        button_proceed.set_sensitive(false);

        let button_open_file = Button::with_label("Open File");

        let button_settings = Button::with_label("Settings");

        let checkbox_sync = gtk4::CheckButton::with_label("Sync");
        checkbox_sync.set_active(false);

        let checkbox_use_model = gtk4::CheckButton::with_label("Enhance image (U-Net)");
        checkbox_use_model.set_active(false);

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

        top_grid.attach(&text_box, 0, 0, 3, 1);

        top_grid.attach(&button_open_file, 3, 0, 1, 1);
        button_open_file.set_hexpand(false);

        let checkbox_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        checkbox_box.append(&checkbox_sync);
        checkbox_box.append(&checkbox_use_model);

        top_grid.attach(&button_settings, 0, 1, 1, 1);
        top_grid.attach(&checkbox_box, 1, 1, 2, 1);
        top_grid.attach(&button_proceed, 3, 1, 1, 1);

        main_vbox.append(&top_grid);

        let picture_widget = gtk4::Picture::new();
        picture_widget.set_hexpand(true);
        picture_widget.set_vexpand(true);

        main_vbox.append(&picture_widget);

        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.set_margin_bottom(12);
        progress_bar.set_margin_start(12);
        progress_bar.set_margin_end(12);
        progress_bar.set_hexpand(true);
        progress_bar.set_vexpand(false);
        progress_bar.set_show_text(true);

        main_vbox.append(&progress_bar);

        window.set_child(Some(&main_vbox));

        //****************
        // Settings window
        //****************
        let settings_window = Window::builder()
            .transient_for(&window)
            .destroy_with_parent(true)
            .default_width(400)
            .default_height(300)
            .resizable(true)
            .modal(true)
            .build();
        settings_window.set_hide_on_close(true);

        // Create a header bar and set it as the titlebar
        let header = HeaderBar::new();
        header.set_show_title_buttons(true);
        settings_window.set_titlebar(Some(&header));

        // Widget - Low pass filter settings
        let low_pass_filter_settings_box = Box::new(gtk4::Orientation::Vertical, 12);
        low_pass_filter_settings_box.set_margin_top(12);
        low_pass_filter_settings_box.set_margin_bottom(12);
        low_pass_filter_settings_box.set_margin_start(12);
        low_pass_filter_settings_box.set_margin_end(12);
        let cutoff_frequency_label = Label::new(Some("Cutoff Frequency (Hz)\n(20-10000)"));
        let cutoff_frequency_spinbutton = SpinButton::builder()
            .adjustment(&gtk4::Adjustment::new(
                5000.0, 20.0, 10000.0, 1.0, 10.0, 0.0,
            ))
            .build();
        cutoff_frequency_spinbutton.set_hexpand(false);
        cutoff_frequency_spinbutton.set_halign(gtk4::Align::Center);
        cutoff_frequency_spinbutton.set_width_request(200);
        cutoff_frequency_spinbutton.set_margin_top(12);
        cutoff_frequency_spinbutton.set_margin_bottom(12);
        cutoff_frequency_spinbutton.set_margin_start(12);
        cutoff_frequency_spinbutton.set_margin_end(12);

        low_pass_filter_settings_box.append(&cutoff_frequency_label);
        low_pass_filter_settings_box.append(&cutoff_frequency_spinbutton);

        // Widget - Envelope detection settings
        let envelope_detection_settings_box = Box::new(gtk4::Orientation::Vertical, 12);
        envelope_detection_settings_box.set_margin_top(12);
        envelope_detection_settings_box.set_margin_bottom(12);
        envelope_detection_settings_box.set_margin_start(12);
        envelope_detection_settings_box.set_margin_end(12);
        let window_size_label = Label::new(Some("Window Size (ms)\n(1-100)"));
        let window_size_spinbutton = SpinButton::builder()
            .adjustment(&gtk4::Adjustment::new(20.0, 1.0, 100.0, 1.0, 10.0, 0.0))
            .build();
        window_size_spinbutton.set_hexpand(false);
        window_size_spinbutton.set_halign(gtk4::Align::Center);
        window_size_spinbutton.set_width_request(200);
        window_size_spinbutton.set_margin_top(12);
        window_size_spinbutton.set_margin_bottom(12);
        window_size_spinbutton.set_margin_start(12);
        window_size_spinbutton.set_margin_end(12);
        let scaling_factor_label = Label::new(Some("Scaling Factor\n(0.1-10)"));
        let scaling_factor_spinbutton = SpinButton::builder()
            .adjustment(&gtk4::Adjustment::new(1.0, 0.1, 10.0, 0.1, 1.0, 0.0))
            .digits(1)
            .build();
        scaling_factor_spinbutton.set_hexpand(false);
        scaling_factor_spinbutton.set_halign(gtk4::Align::Center);
        scaling_factor_spinbutton.set_width_request(200);
        scaling_factor_spinbutton.set_margin_top(12);
        scaling_factor_spinbutton.set_margin_bottom(12);
        scaling_factor_spinbutton.set_margin_start(12);
        scaling_factor_spinbutton.set_margin_end(12);
        envelope_detection_settings_box.append(&window_size_label);
        envelope_detection_settings_box.append(&window_size_spinbutton);
        envelope_detection_settings_box.append(&scaling_factor_label);
        envelope_detection_settings_box.append(&scaling_factor_spinbutton);

        // Widget - Sync apt settings
        let sync_apt_settings_box = Box::new(gtk4::Orientation::Vertical, 12);
        sync_apt_settings_box.set_margin_top(12);
        sync_apt_settings_box.set_margin_bottom(12);
        sync_apt_settings_box.set_margin_start(12);
        sync_apt_settings_box.set_margin_end(12);
        let additional_offset_label = Label::new(Some("Additional Offset (ms) \n(0-500)"));
        let additional_offset_spinbutton = SpinButton::builder()
            .adjustment(&gtk4::Adjustment::new(120.0, 0.0, 500.0, 1.0, 10.0, 0.0))
            .build();
        additional_offset_spinbutton.set_hexpand(false);
        additional_offset_spinbutton.set_halign(gtk4::Align::Center);
        additional_offset_spinbutton.set_width_request(200);
        additional_offset_spinbutton.set_margin_top(12);
        additional_offset_spinbutton.set_margin_bottom(12);
        additional_offset_spinbutton.set_margin_start(12);
        additional_offset_spinbutton.set_margin_end(12);
        sync_apt_settings_box.append(&additional_offset_label);
        sync_apt_settings_box.append(&additional_offset_spinbutton);

        // Widget - Enhance image settings
        let sys = System::new_all();
        let enhance_image_settings_box = Box::new(gtk4::Orientation::Vertical, 12);
        enhance_image_settings_box.set_margin_top(12);
        enhance_image_settings_box.set_margin_bottom(12);
        enhance_image_settings_box.set_margin_start(12);
        enhance_image_settings_box.set_margin_end(12);
        let cpu_threads_label = Label::new(Some(
            &(String::from("CPU Threads\n(1-") + &sys.cpus().len().to_string() + ")"),
        ));
        let cpu_threads_spinbutton = SpinButton::builder()
            .adjustment(&gtk4::Adjustment::new(
                sys.cpus().len() as f64,
                1.0,
                sys.cpus().len() as f64,
                1.0,
                10.0,
                0.0,
            ))
            .build();
        cpu_threads_spinbutton.set_hexpand(false);
        cpu_threads_spinbutton.set_halign(gtk4::Align::Center);
        cpu_threads_spinbutton.set_width_request(200);
        cpu_threads_spinbutton.set_margin_top(12);
        cpu_threads_spinbutton.set_margin_bottom(12);
        cpu_threads_spinbutton.set_margin_start(12);
        cpu_threads_spinbutton.set_margin_end(12);
        enhance_image_settings_box.append(&cpu_threads_label);
        enhance_image_settings_box.append(&cpu_threads_spinbutton);

        // Create a stack and add a couple of pages
        let stack = Stack::new();
        stack.add_titled(
            &low_pass_filter_settings_box,
            Some("low_pass_filter"),
            "Low Pass Filter",
        );
        stack.add_titled(
            &envelope_detection_settings_box,
            Some("envelope_detection"),
            "Envelope Detection",
        );
        stack.add_titled(&sync_apt_settings_box, Some("sync_apt"), "Sync APT");
        stack.add_titled(
            &enhance_image_settings_box,
            Some("enhance_image"),
            "Enhance Image",
        );

        // Create a stack switcher and attach it to the stack
        let stack_switcher = StackSwitcher::new();
        stack_switcher.set_stack(Some(&stack));
        header.set_title_widget(Some(&stack_switcher));

        // Set the stack as the window's child and show the window
        settings_window.set_child(Some(&stack));

        Self {
            window,
            text_box,
            button_proceed,
            button_open_file,
            button_settings,
            checkbox_sync,
            checkbox_use_model,
            picture_widget,
            progress_bar,
            // Settings ui
            settings_window,
            cutoff_frequency_spinbutton,
            additional_offset_spinbutton,
            window_size_spinbutton,
            scaling_factor_spinbutton,
            cpu_threads_spinbutton,
        }
    }

    pub fn present_settings(&self) {
        self.settings_window.present();
    }
}
