use adw::prelude::*;
use adw::{
    Application, ApplicationWindow, HeaderBar, OverlaySplitView, ViewStack, ViewSwitcher,
    ViewSwitcherPolicy,
};
use gtk::{
    Adjustment, Box as GtkBox, Button, ColumnView, ColumnViewColumn, DropDown, Entry, Label,
    Orientation, SpinButton, Text, glib,
};
const APP_ID: &str = "com.ohmm-software.Chop-Chop";

pub mod modeling;
pub mod solvers;

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Build VStack for 'edit' window.
    let edit_window_vsk = GtkBox::new(Orientation::Vertical, 10);
    build_edit_window(&edit_window_vsk);

    // Build 'parent' HStack.
    let parent_hsk = GtkBox::new(Orientation::Vertical, 10);
    parent_hsk.append(&edit_window_vsk);

    let materials_list = ColumnView::builder().build();
    materials_list.append_column(&ColumnViewColumn::builder().title("Column 1").build());
    materials_list.append_column(&ColumnViewColumn::builder().title("Column 2").build());
    materials_list.append_column(&ColumnViewColumn::builder().title("Column 3").build());
    // Need to use a ListItemFactory here?

    let split_view = OverlaySplitView::builder()
        .content(&materials_list)
        .sidebar(&parent_hsk)
        .vexpand(true)
        .build();
    let view_stack = ViewStack::new();
    view_stack.add_titled_with_icon(&split_view, None, "Materials", "document-edit-symbolic");
    view_stack.add_titled_with_icon(
        &Text::builder().text("Parts page").build(),
        None,
        "Parts",
        "emoji-body-symbolic",
    );
    view_stack.add_titled_with_icon(
        &Text::builder().text("Solver page").build(),
        None,
        "Solver",
        "emote-love-symbolic",
    );
    let view_switcher = ViewSwitcher::builder()
        .stack(&view_stack)
        .policy(ViewSwitcherPolicy::Wide)
        .build();
    let header_bar = HeaderBar::builder().title_widget(&view_switcher).build();

    // Build HStack to store Menu buttons.
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.append(&header_bar);
    content.append(&view_stack);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Chop-Chop")
        .content(&content)
        .build();

    // Present window
    window.present();
}

fn build_edit_window(edit_window_vsk: &GtkBox) {
    // Build input sections.
    let description_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(
        &description_input_section,
        &String::from("Description"),
        InputSectionChild::Null,
    );

    let quantity_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(
        &quantity_input_section,
        &String::from("Quantity"),
        InputSectionChild::SpinBox,
    );

    let length_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(
        &length_input_section,
        &String::from("Length"),
        InputSectionChild::DropDownMenu,
    );

    let price_input_section = GtkBox::new(Orientation::Vertical, 10);
    build_input_section(
        &price_input_section,
        &String::from("Price"),
        InputSectionChild::Null,
    );

    // Build 'add' button.
    let add_button = Button::builder().label("Add").build();

    // Build edit window.
    edit_window_vsk.append(&description_input_section);
    edit_window_vsk.append(&quantity_input_section);
    edit_window_vsk.append(&length_input_section);
    edit_window_vsk.append(&price_input_section);
    edit_window_vsk.append(&add_button);
}

fn build_input_section(vsk: &GtkBox, title: &String, child_type: InputSectionChild) {
    // TODO: Use custom enum to allow for dynamic 'child' passing.
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
        }
        InputSectionChild::DropDownMenu => {
            input_section.append(&Entry::new());
            let dropdown = DropDown::from_strings(&["Option 1", "Option 2", "Option 3"]);
            dropdown.set_selected(0);
            input_section.append(&dropdown);
        }
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
    Null,
}
