use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{clone, subclass::InitializingObject};
use gtk::{CompositeTemplate, gio, glib};

use super::supply::SupplyGObject;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/supplies_pane.ui")]
    pub struct SuppliesPane {
        // Entry fields
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

        // Buttons
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub update_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,

        // Column view
        #[template_child]
        pub supplies_view: TemplateChild<gtk::ColumnView>,

        // Model (data store)
        pub supplies: RefCell<Option<gio::ListStore>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SuppliesPane {
        const NAME: &'static str = "ChopChopSuppliesPane";
        type Type = super::SuppliesPane;
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
    impl ObjectImpl for SuppliesPane {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_supplies();
            obj.setup_callbacks();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for SuppliesPane {}

    // Trait shared by GTK boxes
    impl BoxImpl for SuppliesPane {}
}

glib::wrapper! {
    pub struct SuppliesPane(ObjectSubclass<imp::SuppliesPane>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SuppliesPane {
    // Appends a column to a list_model.
    fn append_column_to_list_model(
        &self,
        list_model: &TemplateChild<gtk::ColumnView>,
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

    fn factory_connect_bind_supply(
        &self,
        factory: &gtk::SignalListItemFactory,
        factory_type: FactoryType,
    ) {
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
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
        self.factory_connect_bind_supply(&name_factory, FactoryType::NameFactory);
        self.factory_connect_bind_supply(&material_factory, FactoryType::MaterialFactory);
        self.factory_connect_bind_supply(&max_quantity_factory, FactoryType::MaxQuantityFactory);
        self.factory_connect_bind_supply(&price_factory, FactoryType::PriceFactory);
        self.factory_connect_bind_supply(&length_unit_factory, FactoryType::LengthUnitFactory);
        self.factory_connect_bind_supply(&length_factory, FactoryType::LengthFactory);

        // // Add columns to the supplies view
        self.append_column_to_list_model(&supplies_view, "Name", &name_factory);
        self.append_column_to_list_model(&supplies_view, "Material", &material_factory);
        self.append_column_to_list_model(&supplies_view, "Price", &price_factory);
        self.append_column_to_list_model(&supplies_view, "Quantity", &max_quantity_factory);
        self.append_column_to_list_model(&supplies_view, "Unit", &length_unit_factory);
        self.append_column_to_list_model(&supplies_view, "Length", &length_factory);
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

        self.imp().delete_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                // TODO: Do nothing if there is no active/selected row
                // TODO: Does the selection index always match the index in the model?
                let model = window.imp().supplies_view.model();
                let i = model.unwrap().selection().minimum();
                window.supplies().remove(i);
            }
        ));

        self.imp().update_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.update_supply();
            }
        ));

        self.imp()
            .supplies_view
            .model()
            .unwrap()
            .connect_selection_changed(clone!(
                #[weak(rename_to = window)]
                self,
                move |model, _, _| {
                    let i = model.selection().minimum();
                    let binding = window.supplies().item(i).unwrap();
                    let supply_object = binding.downcast_ref::<SupplyGObject>().unwrap();
                    window.imp().name_field.set_text(&supply_object.name());
                    window
                        .imp()
                        .material_field
                        .set_text(&supply_object.material());
                    window
                        .imp()
                        .price_field
                        .set_text(&supply_object.price().to_string());
                    window
                        .imp()
                        .max_quantity_field
                        .set_value(supply_object.max_quantity() as f64);
                    window
                        .imp()
                        .length_field
                        .set_text(&supply_object.length().to_string());
                    // TODO: Set correct unit type
                }
            ));
    }

    fn supplies(&self) -> gio::ListStore {
        self.imp().supplies.borrow().clone().unwrap()
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

        // Reset widgets
        // self.imp().name_field.set_text("");
        // self.imp().material_field.set_text("");
        // self.imp().price_field.set_text("0.00");
        // self.imp().max_quantity_field.set_value(0.0);
        // self.imp().length_field.set_text("");

        let model = self.imp().supplies_view.model().unwrap();
        model.select_item(model.n_items() - 1, true);
    }

    // TODO: Find a way to combine new_supply and new_parts_supply methods.x
    fn update_supply(&self) {
        // TODO: Do nothing if there is no active/selected row
        // TODO: Does the selection index always match the index in the model?
        let model = self.imp().supplies_view.model().unwrap();
        let i = model.selection().minimum();

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
        self.supplies().remove(i);
        self.supplies().insert(i, &supply);
        model.select_item(i, true);
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
