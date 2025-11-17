use adw::subclass::prelude::*;
use gtk::glib::subclass::InitializingObject;
use gtk::{CompositeTemplate, glib};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/unit_row.ui")]
    pub struct UnitRow {}

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for UnitRow {
        const NAME: &'static str = "ChopChopUnitRow";
        type Type = super::UnitRow;
        type ParentType = adw::ComboRow;

        fn class_init(klass: &mut Self::Class) {
            // Required for CompositeTemplate
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            // Required for CompositeTemplate
            obj.init_template();
        }
    }

    impl ObjectImpl for UnitRow {}
    impl WidgetImpl for UnitRow {}
    impl ListBoxRowImpl for UnitRow {}
    impl PreferencesRowImpl for UnitRow {}
    impl ActionRowImpl for UnitRow {}
    impl ComboRowImpl for UnitRow {}
}

glib::wrapper! {
    pub struct UnitRow(ObjectSubclass<imp::UnitRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, adw::ComboRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl UnitRow {}
