use std::cell::{Cell, RefCell, RefMut};
use std::sync::OnceLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Properties, clone, subclass::InitializingObject, subclass::Signal};
use gtk::{CompositeTemplate, PrintOperationAction::PrintDialog, glib};
use pango::FontDescription;

use super::display::DisplayEngine;
use super::unit_row::UnitRow;
use super::utils::*;
use super::window::Window;
use crate::modeling::Solution;
use crate::size::FractionFormat;
use crate::size::{Size, SizeUnit};
use crate::solvers::{Solver, naive_solver::NaiveSolver};
use crate::utils::{compute_supply_consumption, compute_total_price};

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::SolverPane)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/solver_pane.ui")]
    pub struct SolverPane {
        // Entry fields
        #[template_child]
        pub(super) solver_field: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) blade_unit_field: TemplateChild<UnitRow>,
        #[template_child]
        pub(super) major_blade_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) minor_blade_field: TemplateChild<adw::EntryRow>,

        // Used to switch between the solution view and a placeholder
        #[template_child]
        pub(super) content_stack: TemplateChild<gtk::Stack>,

        // Solution view
        #[template_child]
        pub(super) display_area: TemplateChild<gtk::Box>,

        // User preferences (should be bound to settings values)
        #[property(get, set)]
        pub(super) match_scale: Cell<bool>,
        #[property(get, set)]
        pub(super) display_font: RefCell<String>,
        #[property(get, set)]
        pub(super) print_font: RefCell<String>,
        #[property(get, set)]
        pub(super) size_format: Cell<u32>,
        #[property(get, set)]
        pub(super) size_precision: Cell<u32>,
        #[property(get, set)]
        pub(super) price_precision: Cell<u32>,
        #[property(get, set)]
        pub(super) default_unit: Cell<u32>,

        // Solver result
        pub results: RefCell<Option<Result<Solution, String>>>,

        // Display engine for drawing and printing results
        pub display_engine: RefCell<DisplayEngine>,
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
    #[glib::derived_properties]
    impl ObjectImpl for SolverPane {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_callbacks();
            obj.setup_bindings();
            obj.update_fields();
            obj.update_placeholder();
            obj.validate_all_entries();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            // The parent window can watch this signal to track unsaved changes
            SIGNALS.get_or_init(|| vec![Signal::builder("fields-updated").build()])
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
    pub fn blade_width(&self) -> Size {
        let imp = self.imp();
        Size {
            unit: SizeUnit::from(imp.blade_unit_field.selected()),
            major: parse_positive_fraction(&imp.major_blade_field.text(), true).unwrap(),
            minor: parse_positive_fraction(&imp.minor_blade_field.text(), true).unwrap(),
        }
    }

    pub fn clear_results(&self) {
        self.replace_results(None);
        self.redraw();
    }

    // The Send trait is required because the solver will be sent to the worker thread
    pub fn create_solver(&self) -> Box<dyn Solver + Send> {
        match self.imp().solver_field.selected() {
            0 => Box::new(NaiveSolver {}),
            _ => panic!(),
        }
    }

    pub fn field_data(&self) -> (u32, u32, String, String) {
        let imp = self.imp();
        (
            imp.solver_field.selected(),
            imp.blade_unit_field.selected(),
            imp.major_blade_field.text().to_string(),
            imp.minor_blade_field.text().to_string(),
        )
    }

    // https://github.com/gtk-rs/examples/blob/master/src/bin/printing.rs
    pub fn print_results(&self) {
        let print_operation = gtk::PrintOperation::new();

        // Create outside callbacks in case the user changes the font during the print operation
        let font_description = FontDescription::from_string(&self.print_font());

        // Called right before printing starts, after the user has set options
        print_operation.connect_begin_print(clone!(
            #[weak(rename_to = pane)]
            self,
            #[strong]
            font_description,
            move |operation, context| {
                let mut d = pane.display_engine();
                d.paginate(
                    &context.cairo_context(),
                    &font_description,
                    context.width(),
                    context.height(),
                );
                operation.set_n_pages(d.n_pages() as i32);
            },
        ));

        print_operation.connect_draw_page(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, context, i| {
                let d = pane.display_engine();
                d.draw_page(
                    &context.cairo_context(),
                    &font_description,
                    context.width(),
                    i as usize,
                );
            }
        ));
        print_operation
            .run(PrintDialog, self.root().and_downcast_ref::<Window>())
            .unwrap();
    }

    pub fn redraw(&self) {
        self.draw_results();
        self.update_placeholder();
        self.display_engine().display(&self.imp().display_area);
    }

    pub fn replace_field_data(&self, data: (u32, u32, String, String)) {
        let (solver, blade_unit, major_blade, minor_blade) = data;
        let imp = self.imp();
        imp.solver_field.set_selected(solver);
        imp.blade_unit_field.set_selected(blade_unit);
        imp.major_blade_field.set_text(&major_blade);
        imp.minor_blade_field.set_text(&minor_blade);
    }

    pub fn replace_results(&self, results: Option<Result<Solution, String>>) {
        self.imp().results.replace(results);
    }

    // TODO: Clones data
    pub fn results(&self) -> Option<Result<Solution, String>> {
        self.imp().results.borrow().clone()
    }

    fn display_engine(&self) -> RefMut<'_, DisplayEngine> {
        self.imp().display_engine.borrow_mut()
    }

    fn draw_cut_lists(&self, solution: &Solution) {
        let mut d = self.display_engine();

        let max_length: Option<f64> = if self.match_scale() {
            let mut max = 0.0;
            for sub_solution in solution.values() {
                for supply in sub_solution.supplies.iter() {
                    let length = supply.length.to_meters_f64();
                    if length > max {
                        max = length;
                    }
                }
            }
            if max > 0.0 { Some(max) } else { None }
        } else {
            None
        };

        let format = FractionFormat::from(self.size_format(), self.size_precision());

        let mut i = 1;
        for (material, sub_solution) in solution.iter() {
            for cut_list in sub_solution.cut_lists.iter() {
                d.start_section();
                d.append_header_1(&format!("Cut List {} ({})", i, material.name));
                let supply = &sub_solution.supplies[cut_list.supply_index];
                let mut s = String::new();
                s.push_str(&format!("Repeats: {}\n", cut_list.quantity));
                s.push_str(&format!("Supply \"{}\"\n", supply.name));
                s.push_str(&format!(
                    "Original length {}\n",
                    supply.length.format(&format)
                ));
                s.push_str("Parts to cut:\n");
                for (i, part_index) in cut_list.part_indices.iter().enumerate() {
                    let part = &sub_solution.parts[*part_index];
                    s.push_str(&format!(
                        "\t#{}\t{} ({})",
                        i + 1,
                        part.name,
                        part.length.format(&format)
                    ));
                    if i < cut_list.part_indices.len() - 1 {
                        s.push_str("\n");
                    }
                }
                d.append_paragraph(&s);
                d.append_cut_diagram(cut_list, sub_solution, max_length);
                d.end_section();
                i += 1;
            }
        }
    }

    fn draw_results(&self) {
        self.display_engine().clear();
        match self.imp().results.borrow().as_ref() {
            Some(Ok(solution)) => {
                self.draw_solution(solution);
            }
            Some(Err(message)) => {
                let message = &format!("Solver failed\n{}", message);
                self.display_engine().append_paragraph(message);
            }
            None => {
                self.display_engine().append_paragraph("Solver not yet run");
            }
        }
    }

    fn draw_shopping_list(&self, solution: &Solution) {
        let mut d = self.display_engine();
        d.start_section();
        d.append_header_1("Shopping List");

        let format = FractionFormat::from(self.size_format(), self.size_precision());

        // TODO: Only do this once, not whenever we redraw?
        let consumption = compute_supply_consumption(solution);

        // TODO: Order of material iteration is arbitrary (changes every redraw)
        let mut rows = vec![vec![
            String::from("<b>Material</b>"),
            String::from("<b>Supply</b>"),
            String::from("<b>Length</b>"),
            String::from("<b>Price</b>"),
            String::from("<b>Count</b>"),
            String::from("<b>Total</b>"),
        ]];
        for (material, sub_consumption) in consumption.iter() {
            for (i, consumption) in sub_consumption.iter().enumerate() {
                let supply = &solution[material].supplies[i];
                rows.push(vec![
                    material.name.clone(),
                    supply.name.clone(),
                    supply.length.format(&format),
                    format_price(supply.price, self.price_precision()),
                    consumption.to_string(),
                    format_price(supply.price * (*consumption), self.price_precision()),
                ]);
            }
        }
        d.append_table(
            rows,
            vec![
                gtk::Align::Start,
                gtk::Align::Start,
                gtk::Align::End,
                gtk::Align::End,
                gtk::Align::End,
                gtk::Align::End,
            ],
        );

        d.end_section();
    }

    fn draw_solution(&self, solution: &Solution) {
        self.draw_summary(solution);
        self.draw_shopping_list(solution);
        self.draw_cut_lists(solution);
    }

    fn draw_summary(&self, solution: &Solution) {
        let mut d = self.display_engine();
        d.start_section();
        d.append_header_1("Summary");
        d.append_paragraph("Solution found!");

        // TODO: Only do this once, not whenever we redraw?
        let total_price = compute_total_price(solution);

        d.append_paragraph(&format!(
            "Total price {}",
            format_price(total_price, self.price_precision())
        ));
        d.end_section();
    }

    fn setup_bindings(&self) {
        let imp = self.imp();

        // Show the minor length field only if applicable
        imp.blade_unit_field
            .bind_property("selected", &imp.minor_blade_field.get(), "visible")
            .transform_to(|_, i| Some(SizeUnit::from(i).has_minor()))
            .sync_create()
            .build();

        // Set the title of the length field based on the unit type
        imp.blade_unit_field
            .bind_property("selected", &imp.major_blade_field.get(), "title")
            .transform_to(|_, i| Some(SizeUnit::from(i).major_name()))
            .sync_create()
            .build();
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();
        imp.solver_field.connect_selected_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.signal_fields_updated();
            }
        ));
        imp.blade_unit_field.connect_selected_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.validate_all_entries();
                pane.signal_fields_updated();
            }
        ));
        for field in [&imp.major_blade_field, &imp.minor_blade_field] {
            field.connect_changed(clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.validate_all_entries();
                    pane.signal_fields_updated();
                }
            ));
        }
        for name in [
            "match-scale",
            "display-font",
            "size-format",
            "size-precision",
            "price-precision",
        ] {
            self.connect_notify(Some(name), |pane, _| {
                pane.redraw();
            });
        }
        self.connect_notify(Some("default-unit"), |pane, _| {
            pane.update_fields();
        });
    }

    fn signal_fields_updated(&self) {
        self.emit_by_name::<()>("fields-updated", &[]);
    }

    fn update_fields(&self) {
        let imp = self.imp();
        if imp.major_blade_field.text().as_str().trim().is_empty()
            && imp.minor_blade_field.text().as_str().trim().is_empty()
        {
            imp.blade_unit_field.set_selected(self.default_unit());
        }
    }

    fn update_placeholder(&self) {
        let name = if self.imp().results.borrow().is_none() {
            "placeholder"
        } else {
            "nonempty"
        };
        self.imp().content_stack.set_visible_child_name(name);
    }

    fn use_minor_blade(&self) -> bool {
        SizeUnit::from(self.imp().blade_unit_field.selected()).has_minor()
    }

    fn validate_all_entries(&self) {
        let mut all_valid = true;
        let imp = self.imp();
        all_valid &= validate_entry(&imp.major_blade_field.get(), None, |e| {
            parse_positive_fraction(&e.text(), true).is_ok()
        });
        if self.use_minor_blade() {
            all_valid &= validate_entry(&imp.minor_blade_field.get(), None, |e| {
                parse_positive_fraction(&e.text(), true).is_ok()
            });
        }
        self.action_set_enabled("win.solve", all_valid);
    }
}
