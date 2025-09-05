use std::cell::RefCell;
use std::thread;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Object, clone, subclass::InitializingObject};
use gtk::{ColumnView, CompositeTemplate, PrintOperation, gio, glib};

use super::overlay::Overlay;
use super::parts::PartGObject;
use super::supply::SupplyGObject;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/window.ui")]
    pub struct Window {
        // References to widgets in the supplies pane
        #[template_child]
        pub name_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub material_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub price_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub max_quantity_field: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub length_unit_field: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub length_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub supplies_view: TemplateChild<gtk::ColumnView>,

        // Reference to widgets in the parts pane
        #[template_child]
        pub parts_name_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub parts_material_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub parts_max_quantity_field: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub parts_length_unit_field: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub parts_length_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub parts_add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub parts_view: TemplateChild<gtk::ColumnView>,

        // References to widgets in the solver pane
        #[template_child]
        pub solver_field: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub run_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub print_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,

        // Model (data store) for the supply data
        pub supplies: RefCell<Option<gio::ListStore>>,
        pub parts: RefCell<Option<gio::ListStore>>,
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
            let obj = self.obj();
            obj.setup_supplies();
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
    pub fn new(app: &adw::Application) -> Self {
        Object::builder().property("application", app).build()
    }

    // https://gtk-rs.org/gtk4-rs/git/book/main_event_loop.html#channels
    fn run_solver(&self) {
        let overlay = Overlay::new();
        overlay.set_can_close(false);
        overlay.present(Some(self));

        let (sender, receiver) = async_channel::bounded(1);

        // TODO: Replace this with the actual solver logic
        // TODO: Pass solvers a progress callback
        gio::spawn_blocking(move || {
            let t = 10;
            for i in 0..t {
                let progress = (i as f64) / (t as f64);
                sender.send_blocking(progress).expect("Channel closed");
                thread::sleep(Duration::from_secs(1));
            }
            sender.send_blocking(1.0).expect("Channel closed");
        });

        glib::spawn_future_local(clone!(
            #[weak]
            overlay,
            async move {
                while let Ok(progress) = receiver.recv().await {
                    overlay.update_progress(progress);
                    if progress == 1.0 {
                        overlay.force_close();
                    }
                }
            }
        ));
    }

    // Appends a column to a list_model.
    fn append_column_to_list_model(
        &self,
        list_model: &TemplateChild<ColumnView>,
        title: &str,
        factory: &impl IsA<gtk::ListItemFactory>,
    ) {
        // Add columns to the view
        list_model.append_column(
            &gtk::ColumnViewColumn::builder()
                .title(title)
                .expand(true)
                .factory(factory)
                .build(),
        );
    }

    fn factory_connect_setup(&self, factory: &gtk::SignalListItemFactory) {
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            list_item.set_child(Some(&label));
        });
    }

    fn factory_connect_bind(
        &self,
        factory: &gtk::SignalListItemFactory,
        factory_type: FactoryType,
    ) {
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            // TODO: The below line is getting mad when I click the 'add' button in the parts section.
            let supply_object = list_item.item().and_downcast::<SupplyGObject>().unwrap();
            let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
            match factory_type {
                FactoryType::NameFactory => label.set_label(&supply_object.name()),
                FactoryType::MaterialFactory => label.set_label(&supply_object.material()),
                FactoryType::PriceFactory => label.set_label(&supply_object.price().to_string()),
                FactoryType::MaxQuantityFactory => {
                    label.set_label(&supply_object.max_quantity().to_string())
                }
                FactoryType::LengthUnitFactory => label.set_label(&supply_object.length_unit()),
                FactoryType::LengthFactory => label.set_label(&supply_object.length().to_string()),
            }
        });
    }

    fn setup_supplies(&self) {
        // Create the list model and link it to the column view
        let model = Some(gio::ListStore::new::<SupplyGObject>());
        self.imp().supplies.replace(model);
        let supplies_view = &self.imp().supplies_view;
        let selection = gtk::SingleSelection::new(Some(self.supplies()));
        supplies_view.set_model(Some(&selection));

        // Create the list model and link it to the column view
        let model = Some(gio::ListStore::new::<PartGObject>());
        self.imp().parts.replace(model);
        let parts_view = &self.imp().parts_view;
        let parts_selection = gtk::SingleSelection::new(Some(self.parts()));
        parts_view.set_model(Some(&parts_selection));

        // Create a factory for each column
        let name_factory = gtk::SignalListItemFactory::new();
        let material_factory = gtk::SignalListItemFactory::new();
        let max_quantity_factory = gtk::SignalListItemFactory::new();
        let price_factory = gtk::SignalListItemFactory::new();
        let length_unit_factory = gtk::SignalListItemFactory::new();
        let length_factory = gtk::SignalListItemFactory::new();

        // Callbacks invoked when a new widget needs to be created
        self.factory_connect_setup(&name_factory);
        self.factory_connect_setup(&material_factory);
        self.factory_connect_setup(&max_quantity_factory);
        self.factory_connect_setup(&price_factory);
        self.factory_connect_setup(&length_unit_factory);
        self.factory_connect_setup(&length_factory);

        // Callbacks invoked when an item in the model needs to be bound to a widget
        self.factory_connect_bind(&name_factory, FactoryType::NameFactory);
        self.factory_connect_bind(&material_factory, FactoryType::MaterialFactory);
        self.factory_connect_bind(&max_quantity_factory, FactoryType::MaxQuantityFactory);
        self.factory_connect_bind(&price_factory, FactoryType::PriceFactory);
        self.factory_connect_bind(&length_unit_factory, FactoryType::LengthUnitFactory);
        self.factory_connect_bind(&length_factory, FactoryType::LengthFactory);

        // // Add columns to the supplies view
        self.append_column_to_list_model(&supplies_view, "Name", &name_factory);
        self.append_column_to_list_model(&supplies_view, "Material", &material_factory);
        self.append_column_to_list_model(&supplies_view, "Price", &price_factory);
        self.append_column_to_list_model(&supplies_view, "Quantity", &max_quantity_factory);
        self.append_column_to_list_model(&supplies_view, "Unit", &length_unit_factory);
        self.append_column_to_list_model(&supplies_view, "Length", &length_factory);

        // Add columns to the parts view
        self.append_column_to_list_model(&parts_view, "Name", &name_factory);
        self.append_column_to_list_model(&parts_view, "Material", &material_factory);
        self.append_column_to_list_model(&parts_view, "Quantity", &max_quantity_factory);
        self.append_column_to_list_model(&parts_view, "Unit", &length_unit_factory);
        self.append_column_to_list_model(&parts_view, "Length", &length_factory);
    }

    fn setup_callbacks(&self) {
        // Set up callback for clicking the add button
        self.imp().add_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.new_supply();
            }
        ));

        self.imp().parts_add_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.new_parts_supply();
            }
        ));

        // Set up callback for clicking the run button
        // TODO: Lock UI *immediately* after pressing (currently possible to double-click)
        self.imp().run_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.run_solver();
            }
        ));

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
        self.imp().print_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
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
                    .run(gtk::PrintOperationAction::PrintDialog, Some(&window))
                    .unwrap();
            }
        ));
    }

    fn supplies(&self) -> gio::ListStore {
        self.imp().supplies.borrow().clone().unwrap()
    }

    fn parts(&self) -> gio::ListStore {
        self.imp().parts.borrow().clone().unwrap()
    }

    // TODO: Find a way to combine new_supply and new_parts_supply methods.x
    fn new_supply(&self) {
        // TODO: Get string directly from the combo box?
        let length_unit = String::from(match self.imp().length_unit_field.selected() {
            0 => "Inches",
            1 => "Centimeters",
            2 => "Meters",
            _ => panic!(),
        });

        // TODO: Improve invalid float handling
        let supply = SupplyGObject::new(
            self.imp().name_field.text().to_string(),
            self.imp().material_field.text().to_string(),
            self.imp().price_field.text().parse().unwrap_or(0.0),
            self.imp().max_quantity_field.value() as u32,
            length_unit,
            self.imp().length_field.text().parse().unwrap_or(1.0),
        );
        self.supplies().append(&supply);
    }

    fn new_parts_supply(&self) {
        // TODO: Get string directly from the combo box?
        let length_unit = String::from(match self.imp().parts_length_unit_field.selected() {
            0 => "Inches",
            1 => "Centimeters",
            2 => "Meters",
            _ => panic!(),
        });

        // TODO: Improve invalid float handling
        let supply = PartGObject::new(
            self.imp().parts_name_field.text().to_string(),
            self.imp().parts_material_field.text().to_string(),
            self.imp().parts_max_quantity_field.value() as u32,
            length_unit,
            self.imp().parts_length_field.text().parse().unwrap_or(1.0),
        );
        self.parts().append(&supply);
    }
}

enum FactoryType {
    NameFactory,
    MaterialFactory,
    PriceFactory,
    MaxQuantityFactory,
    LengthUnitFactory,
    LengthFactory,
}
