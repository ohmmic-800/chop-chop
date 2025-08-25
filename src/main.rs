use adw::{
    prelude::*, Application, ApplicationWindow, ComboRow, HeaderBar, OverlaySplitView, SpinRow, ViewStack, ViewSwitcher, ViewSwitcherPolicy,
    EntryRow
};
use gtk::{
    Box as GtkBox, ColumnView, ColumnViewColumn, Orientation, StringList, Text, glib,
    Adjustment, Button, DropDown
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
    let edit_window_vsk = GtkBox::new(Orientation::Vertical, 25);
    edit_window_vsk.set_width_request(250); // Needed to prevent multiline titles in edit window. 
    build_edit_window(&edit_window_vsk);

    // Build 'parent' HStack.
    let parent_hsk = GtkBox::new(Orientation::Vertical, 10);
    parent_hsk.append(&edit_window_vsk);

    let materials_list = ColumnView::builder().build();
    materials_list.append_column(&ColumnViewColumn::builder().title("Column 1").build());
    materials_list.append_column(&ColumnViewColumn::builder().title("Column 2").build());
    materials_list.append_column(&ColumnViewColumn::builder().title("Column 3").build());
    // Need to use a ListItemFactory here?

    let split_view_1 = OverlaySplitView::builder()
        .content(&materials_list)
        .sidebar(&parent_hsk)
        .vexpand(true)
        .min_sidebar_width(250.0)
        .build();
    let view_stack = ViewStack::new();
    view_stack.add_titled_with_icon(&split_view_1, None, "Materials", "document-edit-symbolic");
    view_stack.add_titled_with_icon(
        &Text::builder().text("Parts page").build(),
        None,
        "Parts",
        "emoji-body-symbolic",
    );
    let solver_sidebar = GtkBox::new(Orientation::Vertical, 10);
    solver_sidebar.append(
        &ComboRow::builder()
            .title("Algorithm")
            .subtitle("Optimization method")
            .model(&StringList::new(&[&"Algo1", &"Algo2", &"Algo3"]))
            .selected(0)
            .build(),
    );
    let split_view_3 = OverlaySplitView::builder()
        .content(&Text::builder().text("Solver content").build())
        .sidebar(&solver_sidebar)
        .vexpand(true)
        .min_sidebar_width(250.0)
        .build();
    view_stack.add_titled_with_icon(&split_view_3, None, "Solver", "emote-love-symbolic");
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
        .width_request(750)
        .height_request(500)
        .build();

    // Present window
    window.present();
}

fn build_edit_window(edit_window_vsk: &GtkBox) {
    // Build description entry. 
    let description_entry = EntryRow::new();
    description_entry.set_title("Description");
    description_entry.set_show_apply_button(true);

    // Build Quantity spin box
    let adjustment = Adjustment::new(0.0, 0.0, 1000.0, 1.0, 10.0, 0.0);
    let quantity_row = SpinRow::new(Some(&adjustment), 1.0, 0);
    quantity_row.set_title("Quantity");

    // Build length entry row. 
    let unit_menu_hsk = GtkBox::new(Orientation::Horizontal, 10);
    let unit_menu_entry = EntryRow::new();
    unit_menu_entry.set_title("Length"); // TODO: Split into 2 cells?
    unit_menu_entry.set_show_apply_button(true);

    let options = ["Option 1", "Option 2", "Option 3"];
    let unit_dropdown = DropDown::from_strings(&options);

    // Build length/unit stack. 
    unit_menu_hsk.append(&unit_menu_entry);
    unit_menu_hsk.append(&unit_dropdown);

    // Build price entry
    let price_entry = EntryRow::new();
    price_entry.set_title("Price(USD)");
    price_entry.set_show_apply_button(true);

    // Build add button. 
    let add_button = Button::new();
    add_button.set_label("Add");

    // Add items to the stack. 
    edit_window_vsk.append(&description_entry);
    edit_window_vsk.append(&quantity_row);
    edit_window_vsk.append(&unit_menu_hsk);
    edit_window_vsk.append(&price_entry);
    edit_window_vsk.append(&add_button);
}


