use std::thread::sleep;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, clone, subclass::InitializingObject};
use gtk::{gio, glib};

use super::about_dialog::create_about_dialog;
use super::entry_pane::EntryPane;
use super::solver_overlay::SolverOverlay;
use super::solver_pane::SolverPane;
use super::utils::*;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/window.ui")]
    pub struct Window {
        #[template_child]
        pub supplies_pane: TemplateChild<EntryPane>,
        #[template_child]
        pub parts_pane: TemplateChild<EntryPane>,
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
    impl ObjectImpl for Window {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_actions();
            self.obj().setup_callbacks();
        }
    }

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

    fn setup_actions(&self) {
        let action = gio::ActionEntry::builder("about")
            .activate(|window: &Self, _, _| {
                create_about_dialog().present(Some(window));
            })
            .build();
        self.add_action_entries([action]);
    }

    fn setup_callbacks(&self) {
        // TODO: Lock UI *immediately* after pressing (currently possible to double-click)
        self.imp()
            .solver_pane
            .imp()
            .run_button
            .connect_clicked(clone!(
                #[weak(rename_to = window)]
                self,
                move |_| {
                    window.run_solver();
                }
            ));
    }

    // https://gtk-rs.org/gtk4-rs/git/book/main_event_loop.html#channels
    fn run_solver(&self) {
        let overlay = SolverOverlay::new();
        overlay.present(Some(self));

        let solver = self.imp().solver_pane.create_solver();
        let supplies = parse_supply_entries(self.imp().supplies_pane.entry_data());
        let parts = parse_part_entries(self.imp().parts_pane.entry_data());

        let (progress_sender, progress_receiver) = async_channel::bounded(1);
        let (result_sender, result_receiver) = async_channel::bounded(1);

        gio::spawn_blocking(move || {
            // TODO: Remove (temporary to ensure the dialog and placeholder are working)
            sleep(Duration::from_secs(1));
            progress_sender
                .send_blocking(0.5)
                .expect("Connection closed");
            sleep(Duration::from_secs(1));

            let _ = solver.solve(
                &supplies,
                &parts,
                Some(progress_sender),
                Some(result_sender),
            );
        });

        glib::spawn_future_local(clone!(
            #[weak]
            overlay,
            #[weak(rename_to = window)]
            self,
            async move {
                while let Ok(progress) = progress_receiver.recv().await {
                    overlay.update_progress(progress);
                }
                let results = result_receiver.recv().await.expect("Channel closed");
                window.imp().solver_pane.update_results(results);
                overlay.force_close();
            }
        ));
    }
}
