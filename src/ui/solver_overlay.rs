use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, clone, subclass::InitializingObject};
use gtk::{CompositeTemplate, glib};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/solver_overlay.ui")]
    pub struct SolverOverlay {
        #[template_child]
        pub(super) progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub(super) sub_progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub(super) cancel_button: TemplateChild<gtk::Button>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SolverOverlay {
        const NAME: &'static str = "ChopChopSolverOverlay";
        type Type = super::SolverOverlay;
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
    impl ObjectImpl for SolverOverlay {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_callbacks();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for SolverOverlay {}

    // Trait shared by all Adwaita overlays
    impl AdwDialogImpl for SolverOverlay {}
}

glib::wrapper! {
    pub struct SolverOverlay(ObjectSubclass<imp::SolverOverlay>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl SolverOverlay {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn update_progress(&self, progress: f64) {
        self.imp().progress_bar.set_fraction(progress);
    }

    pub fn update_sub_progress(&self, sub_progress: f64) {
        self.imp().sub_progress_bar.set_fraction(sub_progress);
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
