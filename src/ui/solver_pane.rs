use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{clone, subclass::InitializingObject};
use gtk::{CompositeTemplate, PrintOperationAction::PrintDialog, cairo, glib};
use pangocairo::functions::{create_layout, show_layout};

use super::window::Window;
use crate::solvers::{Solution, Solver, naive_solver::NaiveSolver};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/solver_pane.ui")]
    pub struct SolverPane {
        // Entry fields
        #[template_child]
        pub solver_field: TemplateChild<adw::ComboRow>,

        // Buttons
        #[template_child]
        pub run_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub clear_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub print_button: TemplateChild<gtk::Button>,

        // Used to switch between the column view and a placeholder
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,

        // Solution view
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,

        // Solver result
        pub results: RefCell<Option<Result<Solution, String>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SolverPane {
        const NAME: &'static str = "ChopChopSolverPane";
        type Type = super::SolverPane;
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
    impl ObjectImpl for SolverPane {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_callbacks();
            obj.update_placeholder();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for SolverPane {}

    // Trait shared by GTK boxes
    impl BoxImpl for SolverPane {}
}

glib::wrapper! {
    pub struct SolverPane(ObjectSubclass<imp::SolverPane>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SolverPane {
    pub fn clear_results(&self) {
        self.imp().results.replace(None);
        self.update_placeholder();
        self.imp().drawing_area.queue_draw();
    }

    // The Send trait is required because the solver will be sent to the worker thread
    pub fn create_solver(&self) -> Box<dyn Solver + Send> {
        match self.imp().solver_field.selected() {
            0 => Box::new(NaiveSolver {}),
            _ => panic!(),
        }
    }

    pub fn update_results(&self, results: Result<Solution, String>) {
        self.imp().results.replace(Some(results));
        self.imp().drawing_area.queue_draw();
        self.update_placeholder();
    }

    fn draw_results(&self, c: &cairo::Context, w: f64, h: f64, force_light_bg: bool) {
        // Init pango and set a font
        let p = create_layout(c);
        p.set_font_description(Some(&pango::FontDescription::from_string("sans 14")));

        // Foreground color depends on dark mode and whether we are printing
        if force_light_bg {
            c.set_source_rgb(0.0, 0.0, 0.0);
        } else {
            c.set_source_color(&self.color());
        }

        // TODO: Remove (temporary to demonstrate drawing shapes)
        c.rectangle(20.0, 20.0, w - 40.0, h - 40.0);
        c.stroke().unwrap();
        c.move_to(40.0, 40.0);

        match self.imp().results.borrow().as_ref() {
            Some(Ok(solution)) => {
                Self::draw_solution(solution, c, &p, w, h);
            }
            Some(Err(message)) => {
                p.set_text(&format!("Solver failed\nMessage: {}", message));
                show_layout(c, &p);
            }
            None => {
                p.set_text("Run solver to view solution");
                show_layout(c, &p);
            }
        }
    }

    fn draw_solution(solution: &Solution, c: &cairo::Context, p: &pango::Layout, _w: f64, _h: f64) {
        let mut text = String::from("Solver ran successfully\nResults:\n");
        text.push_str(&format!("Total price: ${:.2}\n", solution.total_price));
        for (i, (cut_list, supply_consumption)) in solution
            .cut_lists
            .iter()
            .zip(solution.supply_consumption.iter())
            .enumerate()
        {
            text.push_str(&format!("\nSupply {}\n", i + 1));
            text.push_str(&format!("Number consumed: {}\n", supply_consumption));
            text.push_str("Cut list (in meters):\n");
            for cut in &cut_list.cuts {
                text.push_str(&format!("{:.10}\n", cut));
            }
        }
        p.set_text(&text);
        show_layout(c, p);
    }

    // https://github.com/gtk-rs/examples/blob/master/src/bin/printing.rs
    fn print_results(&self) {
        let print_operation = gtk::PrintOperation::new();
        print_operation.connect_begin_print(move |print_operation, _| {
            // TODO: Calculate the number of pages dynamically
            print_operation.set_n_pages(1);
        });
        print_operation.connect_draw_page(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, print_context, _| {
                pane.draw_results(
                    &print_context.cairo_context(),
                    print_context.width(),
                    print_context.height(),
                    true,
                );
            }
        ));
        print_operation
            .run(PrintDialog, self.root().and_downcast_ref::<Window>())
            .unwrap();
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();
        imp.drawing_area.set_draw_func(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, cairo, w, h| {
                pane.draw_results(cairo, w as f64, h as f64, false);
            }
        ));
        imp.clear_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.clear_results();
            }
        ));
        imp.print_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.print_results();
            }
        ));
    }

    fn update_placeholder(&self) {
        let name = if self.imp().results.borrow().is_none() {
            "placeholder"
        } else {
            "nonempty"
        };
        self.imp().content_stack.set_visible_child_name(name);
    }
}
