use std::thread::sleep;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use fraction::Fraction;
use gtk::glib::{Object, clone, subclass::InitializingObject};
use gtk::prelude::ButtonExt;
use gtk::{CompositeTemplate, gio, glib};

use super::entry_pane::EntryPane;
use super::solver_overlay::SolverOverlay;
use super::solver_pane::SolverPane;
use crate::modeling::{Part, Supply};
use crate::solvers::Solver;
use crate::solvers::naive_solver::NaiveSolver;

const FEET_TO_METERS_NUM: u64 = 3048;
const FEET_TO_METERS_DEN: u64 = 10000;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
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

    // TODO: Break up
    fn run_solver(&self) {
        let supplies_pane = &self.imp().supplies_pane;
        let supplies_entry_data = supplies_pane.entry_data_vec();
        let mut supplies = Vec::<Supply>::with_capacity(supplies_entry_data.len());
        for entry_data in supplies_entry_data {
            let length = self.length_to_meters(
                supplies_pane.parse_length(&entry_data.length).unwrap(),
                supplies_pane
                    .parse_length_optional(&entry_data.sublength)
                    .unwrap(),
                entry_data.length_unit,
            );
            supplies.push(Supply {
                material: entry_data.material,
                length: length,
                price: supplies_pane.parse_price(&entry_data.price).unwrap(),
                max_quantity: supplies_pane.parse_quantity(&entry_data.quantity).unwrap(),
            });
        }

        let parts_pane = &self.imp().parts_pane;
        let parts_entry_data = parts_pane.entry_data_vec();
        let mut parts = Vec::<Part>::with_capacity(parts_entry_data.len());
        for entry_data in parts_entry_data {
            let length = self.length_to_meters(
                parts_pane.parse_length(&entry_data.length).unwrap(),
                parts_pane
                    .parse_length_optional(&entry_data.sublength)
                    .unwrap(),
                entry_data.length_unit,
            );
            parts.push(Part {
                material: entry_data.material,
                length: length,
                quantity: supplies_pane.parse_quantity(&entry_data.quantity).unwrap(),
            });
        }

        // https://gtk-rs.org/gtk4-rs/git/book/main_event_loop.html#channels

        let overlay = SolverOverlay::new();
        overlay.set_can_close(false);
        overlay.present(Some(self));

        let (progress_sender, progress_receiver) = async_channel::bounded(1);
        let (result_sender, result_receiver) = async_channel::bounded(1);

        // TODO: Replace this with the actual solver logic
        // TODO: Pass solvers a progress callback
        gio::spawn_blocking(move || {
            // TODO: Select correct solver
            let solver = NaiveSolver {};

            // To ensure the dialog and placeholder are working
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
            println!("Solving done!");
        });

        glib::spawn_future_local(clone!(
            #[weak]
            overlay,
            #[weak(rename_to = window)]
            self,
            async move {
                while let Ok(progress) = progress_receiver.recv().await {
                    overlay.update_progress(progress);
                    if progress == 1.0 {
                        overlay.force_close();
                    }
                }
                let result = result_receiver.recv().await.expect("Channel closed");
                window.imp().solver_pane.update_result(result);
            }
        ));
    }

    // TODO: Move this somewhere else
    fn length_to_meters(&self, length: Fraction, sublength: Fraction, unit: u32) -> Fraction {
        let feet_to_meters = Fraction::new(FEET_TO_METERS_NUM, FEET_TO_METERS_DEN);
        match unit {
            0 => feet_to_meters * (length + sublength * 12),
            1 => feet_to_meters * sublength * 12,
            2 => length,
            3 => length / 100,
            _ => panic!(),
        }
    }
}
