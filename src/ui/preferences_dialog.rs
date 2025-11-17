use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, subclass::InitializingObject};
use gtk::{CompositeTemplate, gio::Settings, glib};

use super::{font_row::FontRow, unit_row::UnitRow};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/preferences_dialog.ui")]
    pub struct PreferencesDialog {
        #[template_child]
        pub(super) exit_prompt_entry: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) reopen_last_entry: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) match_scale_entry: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) display_font_entry: TemplateChild<FontRow>,
        #[template_child]
        pub(super) print_font_entry: TemplateChild<FontRow>,
        #[template_child]
        pub(super) size_format_entry: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) size_precision_entry: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(super) price_precision_entry: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(super) deselect_add_entry: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) default_unit_entry: TemplateChild<UnitRow>,
        #[template_child]
        pub(super) default_material_1d_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) default_material_2d_entry: TemplateChild<adw::EntryRow>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesDialog {
        const NAME: &'static str = "ChopChopPreferencesDialog";
        type Type = super::PreferencesDialog;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            // Required for CompositeTemplate
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            // Required for CompositeTemplate
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PreferencesDialog {}

    // Trait shared by all widgets
    impl WidgetImpl for PreferencesDialog {}

    // Trait shared by all Adwaita dialogs
    impl AdwDialogImpl for PreferencesDialog {}

    // Trait shared by all Adwaita preference dialogs
    impl PreferencesDialogImpl for PreferencesDialog {}
}

glib::wrapper! {
    pub struct PreferencesDialog(ObjectSubclass<imp::PreferencesDialog>)
        @extends adw::PreferencesDialog, adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl PreferencesDialog {
    pub fn new(settings: &Settings) -> Self {
        let dialog: Self = Object::builder().build();
        dialog.setup_pref_bindings(settings);
        dialog
    }

    fn setup_pref_bindings(&self, settings: &Settings) {
        let imp = self.imp();
        settings
            .bind("exit-prompt", &imp.exit_prompt_entry.get(), "active")
            .build();
        settings
            .bind("reopen-last", &imp.reopen_last_entry.get(), "active")
            .build();
        settings
            .bind("match-scale", &imp.match_scale_entry.get(), "active")
            .build();
        settings
            .bind(
                "display-font",
                &imp.display_font_entry.get(),
                "font-desc-str",
            )
            .build();
        settings
            .bind("print-font", &imp.print_font_entry.get(), "font-desc-str")
            .build();
        settings
            .bind("size-format", &imp.size_format_entry.get(), "selected")
            .build();
        settings
            .bind("size-precision", &imp.size_precision_entry.get(), "value")
            .build();
        settings
            .bind("price-precision", &imp.price_precision_entry.get(), "value")
            .build();
        settings
            .bind("deselect-add", &imp.deselect_add_entry.get(), "active")
            .build();
        settings
            .bind("default-unit", &imp.default_unit_entry.get(), "selected")
            .build();
        settings
            .bind(
                "default-material-1d",
                &imp.default_material_1d_entry.get(),
                "text",
            )
            .build();
        settings
            .bind(
                "default-material-2d",
                &imp.default_material_2d_entry.get(),
                "text",
            )
            .build();
    }
}
