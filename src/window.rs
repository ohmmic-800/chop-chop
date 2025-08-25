use adw::subclass::prelude::*;
use glib::Object;
use glib::subclass::InitializingObject;
use gtk::{CompositeTemplate, gio, glib};

// Tutorials/examples:
// https://gtk-rs.org/gtk4-rs/git/book/composite_templates.html
// https://github.com/pika-backup/pika-backup/blob/31f6830db2c2beb3b0eea4d7063b7074f363c286/src/ui/widget/app_window.rs
// https://github.com/pika-backup/pika-backup/blob/31f6830db2c2beb3b0eea4d7063b7074f363c286/src/ui/widget/app_window.ui

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/window.ui")]
    pub struct Window {}

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "ChopChopWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
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
        // Create new window
        Object::builder().property("application", app).build()
    }
}
