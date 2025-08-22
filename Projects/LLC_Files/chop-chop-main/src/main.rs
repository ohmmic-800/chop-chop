use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Orientation, Entry, Label, Adjustment, SpinButton, DropDown}; // TODO: Remove unneeded use statements. 
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
    build_menu_bar(&top_menu_hsk);

    // Build VStack for 'edit' window. 
    let edit_window_vsk = GtkBox::new(Orientation::Vertical, 10);
    build_edit_window(&edit_window_vsk);

    // Build 'parent' HStack.
    let parent_hsk = GtkBox::new(Orientation::Vertical, 10);
    parent_hsk.append(&top_menu_hsk);
    parent_hsk.append(&edit_window_vsk);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&parent_hsk)
        .build();

    // Present window
    window.present();
}

fn build_menu_bar(top_menu_hsk: &GtkBox) {
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
}

fn build_edit_window(edit_window_vsk: &GtkBox) {
    // Build input sections. 
    let description_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(&description_input_section, &String::from("Description"), InputSectionChild::Null);

    let quantity_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(&quantity_input_section, &String::from("Quantity"), InputSectionChild::SpinBox);

    let length_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(&length_input_section, &String::from("Length"), InputSectionChild::DropDownMenu);

    let price_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(&price_input_section, &String::from("Price"), InputSectionChild::Null);

    // Build 'add' button. 
    let add_button = Button::builder()
    .label("Add")
    .build();

    // Build edit window. 
    edit_window_vsk.append(&description_input_section);
    edit_window_vsk.append(&quantity_input_section);
    edit_window_vsk.append(&length_input_section);
    edit_window_vsk.append(&price_input_section);
    edit_window_vsk.append(&add_button);
}

fn build_input_section(vsk: &GtkBox, title: &String, child_type: InputSectionChild) { // TODO: Use custom enum to allow for dynamic 'child' passing. 
    let label = Label::new(Some(title));
    
    // Build entry box area. 
    let input_section = GtkBox::new(Orientation::Horizontal, 10);
    input_section.append(&Entry::new());

    // Add child widget when applicable. 
    match child_type {
        InputSectionChild::SpinBox => {
            let adjustment = Adjustment::new(0.0, 0.0, f64::MAX, 1.0, 10.0, 0.0);
            let spin = SpinButton::new(Some(&adjustment), 1.0, 0); 
            input_section.append(&spin);
        },
        InputSectionChild::DropDownMenu => {
            input_section.append(&Entry::new());
            let dropdown = DropDown::from_strings(&["Option 1", "Option 2", "Option 3"]);
                    dropdown.set_selected(0);
            input_section.append(&dropdown);

        },
        InputSectionChild::Null => (),
    };

    // Put all the pieces together. 
    vsk.append(&label);
    vsk.append(&input_section);
}

// This enum represents possible child types for a input_section. 
enum InputSectionChild {
    SpinBox, 
    DropDownMenu,
    Null
}