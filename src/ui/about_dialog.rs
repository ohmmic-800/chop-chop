// TODO: Update with real "Support Us" link
pub fn create_about_dialog() -> adw::AboutDialog {
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
