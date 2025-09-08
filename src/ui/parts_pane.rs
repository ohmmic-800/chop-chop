use std::cell::RefCell;

use adw::subclass::prelude::*;
use gtk::glib::subclass::InitializingObject;
use gtk::{CompositeTemplate, gio, glib};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/parts_pane.ui")]
    pub struct PartsPane {
        // Entry fields
        #[template_child]
        pub name_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub material_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub max_quantity_field: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub length_unit_field: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub length_field: TemplateChild<adw::EntryRow>,

        // Buttons
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        // Column view
        #[template_child]
        pub parts_view: TemplateChild<gtk::ColumnView>,

        // Model (data store)
        pub parts: RefCell<Option<gio::ListStore>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PartsPane {
        const NAME: &'static str = "ChopChopPartsPane";
        type Type = super::PartsPane;
        type ParentType = gtk::Box;

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
    impl ObjectImpl for PartsPane {}

    // Trait shared by all widgets
    impl WidgetImpl for PartsPane {}

    // Trait shared by GTK boxes
    impl BoxImpl for PartsPane {}
}

glib::wrapper! {
    pub struct PartsPane(ObjectSubclass<imp::PartsPane>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

// TODO: Copy code from supplies pane after optimizing/refining
impl PartsPane {}
