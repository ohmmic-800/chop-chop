use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Orientation}; // TODO: Remove unneeded use statements. 
const APP_ID: &str = "org.gtk_rs.HelloWorld3";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {

    // Build HStack to store Menu buttons.
    let top_menu_hsk = GtkBox::new(Orientation::Horizontal, 10);

    // Create Menu Buttons.
    let materials_menu_button = Button::builder()
    .label("Materials")
    .build();

    let cuts_menu_button = Button::builder()
    .label("Cuts")
    .build();

    let solver_menu_button = Button::builder()
    .label("Solver")
    .build();

    let exit_button = Button::builder()
    .label("X")// TODO: Use actual 'exit' char here. 
    .build();

    // Create menu spacers. 
    let top_menu_spacer_left = GtkBox::new(Orientation::Horizontal, 0);
    top_menu_spacer_left.set_width_request(50);

    let top_menu_spacer_right = GtkBox::new(Orientation::Horizontal, 0);
    top_menu_spacer_right.set_width_request(50);


    // Build top menu hstack. 
    top_menu_hsk.append(&top_menu_spacer_right);
    top_menu_hsk.append(&materials_menu_button);
    top_menu_hsk.append(&cuts_menu_button);
    top_menu_hsk.append(&solver_menu_button);
    top_menu_hsk.append(&top_menu_spacer_left);
    top_menu_hsk.append(&exit_button);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&top_menu_hsk)
        .build();

    // Present window
    window.present();
}
