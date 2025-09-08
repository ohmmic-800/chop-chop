use adw::subclass::prelude::*;
use gtk::glib::{Object, subclass::InitializingObject};
use gtk::{CompositeTemplate, gio, glib};

use super::parts_pane::PartsPane;
use super::solver_pane::SolverPane;
use super::supplies_pane::SuppliesPane;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/window.ui")]
    pub struct Window {
        #[template_child]
        pub supplies_pane: TemplateChild<SuppliesPane>,
        #[template_child]
        pub parts_pane: TemplateChild<PartsPane>,
        #[template_child]
        pub solver_pane: TemplateChild<SolverPane>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "ChopChopWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

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
    impl ObjectImpl for Window {}

    // Trait shared by all widgets
    impl WidgetImpl for Window {}

    // Trait shared by all windows
    impl WindowImpl for Window {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for Window {}

    // Trait shared by all Adwaita application windows
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        Object::builder().property("application", app).build()
    }
}
