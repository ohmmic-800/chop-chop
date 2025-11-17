use std::cell::{Cell, RefCell};
use std::fs::File;
use std::path::PathBuf;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, Properties, clone, closure_local, subclass::InitializingObject};
use gtk::{CompositeTemplate, gio, gio::ListStore, gio::Settings, glib};

use super::dialogs::{
    about_dialog, open_failed_dialog, save_failed_dialog, unsaved_changes_dialog,
};
use super::entry_pane::EntryPane;
use super::preferences_dialog::PreferencesDialog;
use super::solver_overlay::SolverOverlay;
use super::solver_pane::SolverPane;
use super::utils::*;
use crate::APP_ID;
use crate::solvers::Message;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::Window)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/window.ui")]
    pub struct Window {
        #[template_child]
        pub(super) filename_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) unsaved_indicator: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub(super) supplies_pane: TemplateChild<EntryPane>,
        #[template_child]
        pub(super) parts_pane: TemplateChild<EntryPane>,
        #[template_child]
        pub(super) solver_pane: TemplateChild<SolverPane>,

        // Whether there are unsaved changes
        #[property(get, set)]
        pub(super) unsaved_changes: Cell<bool>,

        // Filepath of the active project
        #[property(get, set)]
        pub(super) project_filepath: RefCell<Option<String>>,

        // App settings
        pub(super) settings: RefCell<Option<Settings>>,
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
    #[glib::derived_properties]
    impl ObjectImpl for Window {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_settings();
            obj.setup_bindings();
            obj.setup_actions();
            obj.setup_callbacks();
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
    pub fn new(app: &adw::Application, reopen_last: bool) -> Self {
        let window: Self = Object::builder().property("application", app).build();
        let file_path = PathBuf::from(window.settings().string("last-project"));
        if reopen_last && window.settings().boolean("reopen-last") && file_path.exists() {
            window.open_project(&file_path);
        }
        window
    }

    pub fn settings(&self) -> Settings {
        self.imp().settings.borrow().clone().unwrap()
    }

    fn close_dialog(&self) {
        let dialog = unsaved_changes_dialog();
        dialog.connect_response(
            None,
            clone!(
                #[weak(rename_to = window)]
                self,
                move |_, response| {
                    match response {
                        "discard" => {
                            window.set_unsaved_changes(false);
                            window.close();
                        }
                        "save" => {
                            window.save_dialog(true);
                        }
                        _ => (),
                    };
                }
            ),
        );
        dialog.present(Some(self));
    }

    fn open_dialog(&self) {
        let filter_list = ListStore::new::<gtk::FileFilter>();

        let filter = gtk::FileFilter::new();
        filter.set_name(Some("JSON files"));
        filter.add_suffix("json");
        filter_list.append(&filter);

        let filter = gtk::FileFilter::new();
        filter.set_name(Some("All files"));
        filter.add_pattern("*");
        filter_list.append(&filter);

        let file_chooser = gtk::FileDialog::builder().filters(&filter_list).build();
        file_chooser.open(
            Some(self),
            None::<&gio::Cancellable>,
            clone!(
                #[weak(rename_to = window)]
                self,
                move |a| {
                    if let Ok(file) = a
                        && let Some(file_path) = file.path()
                    {
                        window.open_project(&file_path);
                    }
                }
            ),
        );
    }

    fn open_project(&self, file_path: &PathBuf) {
        let file = match File::open(&file_path) {
            Ok(file) => file,
            Err(_) => {
                open_failed_dialog(file_path).present(Some(self));
                return;
            }
        };
        let (supply_entries, part_entries, results, solver_field_data) =
            match serde_json::from_reader(file) {
                Ok(data) => data,
                Err(_) => {
                    open_failed_dialog(file_path).present(Some(self));
                    return;
                }
            };
        let imp = self.imp();
        imp.supplies_pane.replace_entry_data(supply_entries);
        imp.parts_pane.replace_entry_data(part_entries);
        imp.solver_pane.replace_results(unflatten_results(results));
        imp.solver_pane.replace_field_data(solver_field_data);

        // TODO: When might to_str() fail?
        self.set_project_filepath(file_path.to_str().unwrap());
        self.update_last_project();
        self.set_unsaved_changes(false);
        imp.solver_pane.redraw();
    }

    // https://gtk-rs.org/gtk4-rs/git/book/main_event_loop.html#channels
    fn run_solver(&self) {
        let overlay = SolverOverlay::new();
        overlay.present(Some(self));

        let imp = self.imp();
        let solver = imp.solver_pane.create_solver();
        let problem = generate_problem(
            imp.supplies_pane.entry_data(),
            imp.parts_pane.entry_data(),
            imp.solver_pane.blade_width(),
        );
        let (sender, receiver) = async_channel::bounded(1);

        // TODO: Pressing "Cancel" will not stop this thread
        gio::spawn_blocking(clone!(
            #[strong]
            problem,
            move || {
                let _ = solver.solve(problem, Some(sender));
            }
        ));

        glib::spawn_future_local(clone!(
            #[weak]
            overlay,
            #[weak(rename_to = window)]
            self,
            async move {
                let imp = window.imp();
                while let Ok(message) = receiver.recv().await {
                    match message {
                        Message::Progress(progress) => {
                            overlay.update_progress(progress);
                        }
                        Message::SubProgress(sub_progress) => {
                            overlay.update_sub_progress(sub_progress);
                        }
                        Message::Results(results) => {
                            imp.solver_pane.replace_results(Some(results));
                            window.set_unsaved_changes(true);
                            imp.solver_pane.redraw();
                        }
                    }
                }
                overlay.force_close();
            }
        ));
    }

    fn save_dialog(&self, close_on_success: bool) {
        let file_chooser = gtk::FileDialog::builder()
            .initial_name("Project.json")
            .build();
        file_chooser.save(
            Some(self),
            None::<&gio::Cancellable>,
            clone!(
                #[weak(rename_to = window)]
                self,
                move |a| {
                    if let Ok(file) = a
                        && let Some(file_path) = file.path()
                    {
                        window.save_project(&file_path, close_on_success);
                    }
                }
            ),
        );
    }

    fn save_project(&self, file_path: &PathBuf, close_on_success: bool) {
        let file = match File::create(&file_path) {
            Ok(file) => file,
            Err(_) => {
                save_failed_dialog(file_path).present(Some(self));
                return;
            }
        };

        // TODO: These accessor methods return cloned data (wasteful)
        let imp = self.imp();
        let state = (
            imp.supplies_pane.entry_data(),
            imp.parts_pane.entry_data(),
            flatten_results(imp.solver_pane.results()),
            imp.solver_pane.field_data(),
        );
        if serde_json::to_writer_pretty(file, &state).is_err() {
            save_failed_dialog(file_path).present(Some(self));
            return;
        }

        // TODO: When might to_str() fail?
        self.set_project_filepath(file_path.to_str().unwrap());
        self.update_last_project();
        self.set_unsaved_changes(false);
        if close_on_success {
            self.close();
        }
    }

    fn setup_actions(&self) {
        let open_action = gio::ActionEntry::builder("open")
            .activate(|window: &Self, _, _| {
                window.open_dialog();
            })
            .build();
        let save_action = gio::ActionEntry::builder("save")
            .activate(|window: &Self, _, _| match window.project_filepath() {
                Some(file_path) => window.save_project(&PathBuf::from(file_path), false),
                None => window.save_dialog(false),
            })
            .build();
        let save_as_action = gio::ActionEntry::builder("save-as")
            .activate(|window: &Self, _, _| {
                window.save_dialog(false);
            })
            .build();
        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(|window: &Self, _, _| {
                PreferencesDialog::new(&window.settings()).present(Some(window));
            })
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(|window: &Self, _, _| {
                about_dialog().present(Some(window));
            })
            .build();
        let close_action = gio::ActionEntry::builder("close")
            .activate(|window: &Self, _, _| {
                window.close();
            })
            .build();
        let solve_action = gio::ActionEntry::builder("solve")
            .activate(|window: &Self, _, _| {
                window.run_solver();
            })
            .build();
        let print_action = gio::ActionEntry::builder("print")
            .activate(|window: &Self, _, _| {
                window.imp().solver_pane.print_results();
            })
            .build();
        let clear_action = gio::ActionEntry::builder("clear")
            .activate(|window: &Self, _, _| {
                window.imp().solver_pane.clear_results();
            })
            .build();
        self.add_action_entries([
            open_action,
            save_action,
            save_as_action,
            preferences_action,
            about_action,
            close_action,
            solve_action,
            print_action,
            clear_action,
        ]);
    }

    fn setup_bindings(&self) {
        let imp = self.imp();
        let settings = self.settings();
        self.bind_property("unsaved-changes", &imp.unsaved_indicator.get(), "visible")
            .sync_create()
            .build();
        self.bind_property("project-filepath", &imp.filename_label.get(), "label")
            .transform_to(|_, p: Option<&str>| match p {
                // Unwrap is safe because split yields >= 1 substrings
                Some(f) => Some(f.split(std::path::MAIN_SEPARATOR).last().unwrap()),
                None => Some("Untitled"),
            })
            .build();
        self.bind_property(
            "project-filepath",
            &imp.filename_label.get(),
            "tooltip-text",
        )
        .build();
        for pane in [&imp.parts_pane.get(), &imp.supplies_pane.get()] {
            settings.bind("size-format", pane, "size-format").build();
            settings
                .bind("size-precision", pane, "size-precision")
                .build();
            settings
                .bind("price-precision", pane, "price-precision")
                .build();
            settings.bind("deselect-add", pane, "deselect-add").build();
            settings.bind("default-unit", pane, "default-unit").build();
            settings
                .bind("default-material-1d", pane, "default-material-1d")
                .build();
            settings
                .bind("default-material-2d", pane, "default-material-2d")
                .build();
            pane.connect_closure(
                "entries-updated",
                false,
                closure_local!(
                    #[weak(rename_to = window)]
                    self,
                    move |_: EntryPane| {
                        window.set_unsaved_changes(true);
                    }
                ),
            );
        }
        let pane = &imp.solver_pane.get();
        settings.bind("match-scale", pane, "match-scale").build();
        settings.bind("display-font", pane, "display-font").build();
        settings.bind("print-font", pane, "print-font").build();
        settings.bind("size-format", pane, "size-format").build();
        settings
            .bind("size-precision", pane, "size-precision")
            .build();
        settings
            .bind("price-precision", pane, "price-precision")
            .build();
        settings.bind("default-unit", pane, "default-unit").build();
        pane.connect_closure(
            "fields-updated",
            false,
            closure_local!(
                #[weak(rename_to = window)]
                self,
                move |_: SolverPane| {
                    window.set_unsaved_changes(true);
                }
            ),
        );
    }

    fn setup_callbacks(&self) {
        self.connect_close_request(|window| {
            if window.settings().boolean("exit-prompt") && window.unsaved_changes() {
                window.close_dialog();
                glib::Propagation::Stop
            } else {
                window.update_last_project();
                glib::Propagation::Proceed
            }
        });
    }

    fn setup_settings(&self) {
        let settings = Some(Settings::new(APP_ID));
        self.imp().settings.replace(settings);
    }

    fn update_last_project(&self) {
        self.settings()
            .set("last-project", self.project_filepath().unwrap_or_default())
            .expect("Failed to update the project filepath");
    }
}
