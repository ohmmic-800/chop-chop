use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, Properties, subclass::InitializingObject};
use gtk::{CompositeTemplate, glib};
use pango::FontDescription;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::FontRow)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/font_row.ui")]
    pub struct FontRow {
        // Button that summons a font dialog
        #[template_child]
        pub(super) font_button: TemplateChild<gtk::FontDialogButton>,

        // String version of font description
        #[property(get, set)]
        pub(super) font_desc_str: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for FontRow {
        const NAME: &'static str = "ChopChopFontEntry";
        type Type = super::FontRow;
        type ParentType = adw::ActionRow;

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
    #[glib::derived_properties]
    impl ObjectImpl for FontRow {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_bindings();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for FontRow {}

    // Trait shared by all list box rows
    impl ListBoxRowImpl for FontRow {}

    // Traid shared by all Adwaita preference rows
    impl PreferencesRowImpl for FontRow {}

    // Trait shared by all Adwaita action rows
    impl ActionRowImpl for FontRow {}
}

glib::wrapper! {
    pub struct FontRow(ObjectSubclass<imp::FontRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl FontRow {
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup_bindings(&self) {
        let imp = self.imp();
        self.bind_property("font-desc-str", &imp.font_button.get(), "font-desc")
            .transform_to(|_, s| Some(FontDescription::from_string(s)))
            .transform_from(|_, d: FontDescription| Some(d.to_string()))
            .sync_create()
            .bidirectional()
            .build();
    }
}
