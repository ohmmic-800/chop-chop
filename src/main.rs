mod modeling;
mod solvers;
mod ui;
mod units;

use adw::Application;
use adw::prelude::*;
use gtk::{CssProvider, gdk::Display, gio, glib};
use ui::window::Window;

const APP_ID: &str = "com.ohmm-software.Chop-Chop";

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to the "startup" and "activate" signals
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    // Set up application-global actions and keybindings
    setup_actions(&app);
    setup_accels(&app);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    window.present();
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("styles/style.css"));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_actions(app: &Application) {
    let quit_action = gio::ActionEntry::builder("quit")
        .activate(|app: &Application, _, _| app.quit())
        .build();
    app.add_action_entries([quit_action]);
}

fn setup_accels(app: &Application) {
    app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
    app.set_accels_for_action("win.close", &["<Ctrl>W"]);
}
