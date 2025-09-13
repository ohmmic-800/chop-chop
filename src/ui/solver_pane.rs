use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::subclass::InitializingObject;
use gtk::{CompositeTemplate, PrintOperation, glib};

use crate::solvers::Solution;

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
        pub print_button: TemplateChild<gtk::Button>,

        // Solution view
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,

        // Solver result
        pub result: RefCell<Option<Result<Solution, String>>>,
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

// TODO: Copy code from supplies pane after optimizing/refining
impl SolverPane {
    pub fn update_result(&self, result: Result<Solution, String>) {
        self.imp().result.replace(Some(result));
    }

    fn setup_callbacks(&self) {
        // TODO: Ensure output looks the same when printed
        // Has something to do with resolution (display units)

        // TODO: Print button in bottom adwaita toolbar alongside delete button?

        self.imp()
            .drawing_area
            .set_draw_func(move |_area, cairo, w, h| {
                //Initi pango and set a font
                let font_description = pango::FontDescription::from_string("sans 14");
                let pango_layout = pangocairo::functions::create_layout(cairo);
                pango_layout.set_font_description(Option::from(&font_description));

                cairo.set_source_rgb(1.0, 0.5, 0.5);
                cairo.rectangle(5.0, 5.0, (w as f64) - 10.0, (h as f64) - 10.0);
                cairo.stroke().unwrap();

                // Draw text1
                pango_layout.set_text("Hello");
                cairo.move_to(10.0, 10.0);
                pangocairo::functions::show_layout(&cairo, &pango_layout);

                //Draw text2 below text1
                pango_layout.set_text("World");
                cairo.rel_move_to(0.0, 20.0);
                pangocairo::functions::show_layout(&cairo, &pango_layout);
            });

        // Based on this example:
        // https://github.com/gtk-rs/examples/blob/master/src/bin/printing.rs
        self.imp().print_button.connect_clicked(move |_| {
            // TODO: Some parts of the dialog may be disablable using builder options
            let print_operation = PrintOperation::new();

            print_operation.connect_begin_print(move |print_operation, _| {
                // This sets the number of pages of the document.
                // You most likely will calculate this, but for this example
                // it's hardcoded as 1
                print_operation.set_n_pages(1);
            });

            print_operation.connect_draw_page(move |_, print_context, _| {
                let cairo = print_context.cairo_context();

                let w = print_context.width();
                let h = print_context.height();

                //Initi pango and set a font
                let font_description = pango::FontDescription::from_string("sans 14");
                let pango_layout = pangocairo::functions::create_layout(&cairo);

                // let pango_layout = print_context.create_pango_layout();
                pango_layout.set_font_description(Option::from(&font_description));

                cairo.set_source_rgb(1.0, 0.5, 0.5);
                cairo.rectangle(5.0, 5.0, w - 10.0, h - 10.0);
                cairo.stroke().unwrap();

                // Draw text1
                pango_layout.set_text("Hello");
                cairo.move_to(10.0, 10.0);
                pangocairo::functions::show_layout(&cairo, &pango_layout);

                //Draw text2 below text1
                pango_layout.set_text("World");
                cairo.rel_move_to(0.0, 20.0);
                pangocairo::functions::show_layout(&cairo, &pango_layout);
            });

            print_operation
                .run(
                    gtk::PrintOperationAction::PrintDialog,
                    Option::<&gtk::ApplicationWindow>::None,
                )
                .unwrap();
        });
    }
}
