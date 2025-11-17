use std::path::PathBuf;

use adw::prelude::*;

// TODO: Update with real "Support Us" link
pub fn about_dialog() -> adw::AboutDialog {
    let dialog = adw::AboutDialog::builder()
        .application_icon("tree-circle-symbolic")
        .application_name("Chop-Chop")
        .version("0.0.0")
        .website("https://ohmm-software.com")
        .issue_url("https://github.com/ohmm-software/chop-chop/issues")
        .developer_name("OHMM Software")
        .developers([
            "Matthew Dutson <matt@ohmm-software.com>",
            "Samuel Dutson <sam@ohmm-software.com>",
        ])
        .copyright("Copyright © 2025 Matthew Dutson and Samuel Dutson")
        .license_type(gtk::License::Gpl30)
        .comments("Thanks to (the lovely) Itzel Estrella for the name. And thanks as always to Lex de Azevedo and Uwe Rosenberg.")
        .build();
    dialog.add_link("Support Us", "https://ohmm-software.com");
    dialog
}

pub fn open_failed_dialog(file_path: &PathBuf) -> adw::AlertDialog {
    let dialog = adw::AlertDialog::builder()
        .heading("Open Failed")
        .body(&format!("Failed to open \"{}\"", file_path.display()))
        .build();
    dialog.add_response("okay", "Okay");
    dialog.set_default_response(Some("okay"));
    dialog.set_close_response("okay");
    dialog
}

pub fn save_failed_dialog(file_path: &PathBuf) -> adw::AlertDialog {
    let dialog = adw::AlertDialog::builder()
        .heading("Save Failed")
        .body(&format!("Failed to save to \"{}\"", file_path.display()))
        .build();
    dialog.add_response("okay", "Okay");
    dialog.set_default_response(Some("okay"));
    dialog.set_close_response("okay");
    dialog
}

pub fn unsaved_changes_dialog() -> adw::AlertDialog {
    let dialog = adw::AlertDialog::builder()
        .heading("Discard Changes?")
        .body("You have unsaved changes. Do you want to exit without saving?")
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("discard", "Discard");
    dialog.add_response("save", "Save");
    dialog.set_default_response(Some("save"));
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("cancel", adw::ResponseAppearance::Default);
    dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dialog
}
