use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, clone, subclass::InitializingObject};
use gtk::{CompositeTemplate, glib};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/overlay.ui")]
    pub struct Overlay {
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Overlay {
        const NAME: &'static str = "ChopChopOverlay";
        type Type = super::Overlay;
        type ParentType = adw::Dialog;

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
    impl ObjectImpl for Overlay {
        // Called when the object is constructed
        fn constructed(&self) {
            self.obj().setup_callbacks();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for Overlay {}

    // Trait shared by all Adwaita application windows
    impl AdwDialogImpl for Overlay {}
}

glib::wrapper! {
    pub struct Overlay(ObjectSubclass<imp::Overlay>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl Overlay {
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup_callbacks(&self) {
        self.imp().cancel_button.connect_clicked(clone!(
            #[weak(rename_to = overlay)]
            self,
            move |_| {
                overlay.force_close();
            }
        ));
    }
}
