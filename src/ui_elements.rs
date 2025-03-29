use gtk4::{prelude::*, Button, Entry, Picture, ProgressBar, CheckButton, ApplicationWindow};

pub struct UiElements {
    pub window: ApplicationWindow,
    pub text_box: Entry,
    pub button_proceed: Button,
    pub button_open_file: Button,
    pub checkbox_sync: CheckButton,
    pub checkbox_use_model: CheckButton,
    pub picture_widget: Picture,
    pub progress_bar: ProgressBar,
    // Other UI elements as needed
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

        top_grid.attach(&text_box, 0, 0, 2, 1);

        top_grid.attach(&button_open_file, 2, 0, 1, 1);
        button_open_file.set_hexpand(false);
    
        let checkbox_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        checkbox_box.append(&checkbox_sync);
        checkbox_box.append(&checkbox_use_model);
    
        top_grid.attach(&button_proceed, 2, 1, 1, 1);
        top_grid.attach(&checkbox_box, 0, 1, 2, 1);
    
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

        // Additional UI construction can be handled here.

        Self {
            window,
            text_box,
            button_proceed,
            button_open_file,
            checkbox_sync,
            checkbox_use_model,
            picture_widget,
            progress_bar,
        }
    }
}