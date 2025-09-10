use std::cell::RefCell;
use std::str::FromStr;

use adw::prelude::*;
use adw::subclass::prelude::*;
use fraction::Fraction;
use gtk::glib::{clone, subclass::InitializingObject};
use gtk::{gio::ListStore, glib};

use super::supply::SupplyGObject;

enum FieldType {
    String,
    F64,
    U32,
    Fraction,
}

mod imp {
    use super::*;

    // Object holding the state
    #[derive(gtk::CompositeTemplate, Default)]
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
        pub update_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,

        // Used to switch between the column view and a placeholder
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,

        // Column view
        #[template_child]
        pub supplies_view: TemplateChild<gtk::ColumnView>,

        // Model (data store)
        pub supplies: RefCell<Option<ListStore>>,
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
            self.obj().setup_column_view();
            self.obj().setup_callbacks();
            self.obj().update_content_stack();

            // Otherwise higlighting is only applied after the first edit to a validated field
            self.obj().validate_fields();
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
    fn add_supply(&self) {
        self.supplies().append(&self.parse_all_fields());

        // Select the item we just added
        let selection_model = self.selection_model();
        selection_model.select_item(selection_model.n_items() - 1, true);
    }

    fn delete_supply(&self) {
        let selection = self.selection_model().selection();
        if !selection.is_empty() {
            self.supplies().remove(selection.minimum());
        }
    }

    /// Panics if parse_length(text) returns an error
    fn format_fraction(text: &str) -> String {
        // TODO: Make format configurable (original format, fraction, mixed, or decimal)
        // May require making this a method
        // https://docs.rs/fraction/latest/fraction/#format-convert-to-string
        format!("{:.10}", Self::parse_length(text).unwrap())
    }

    fn highlight_field(field: &adw::EntryRow, is_valid: bool) {
        if is_valid {
            field.remove_css_class("invalid-entry");
        } else {
            field.add_css_class("invalid-entry");
        }
    }

    fn parse_all_fields(&self) -> SupplyGObject {
        // TODO: Manual match is bad
        let length_unit = String::from(match self.imp().length_unit_field.selected() {
            0 => "Inches",
            1 => "Centimeters",
            2 => "Meters",
            _ => panic!(),
        });
        SupplyGObject::new(
            self.imp().name_field.text().to_string(),
            self.imp().material_field.text().to_string(),
            Self::parse_price(&self.imp().price_field.text()).unwrap(),
            self.imp().max_quantity_field.value() as u32,
            length_unit,
            self.imp().length_field.text().to_string(),
        )
    }

    /// Allows for an arbitrary number of fraction terms (not limited to <= 2)
    fn parse_length(text: &str) -> Result<Fraction, ()> {
        let text = text.trim();
        if text.is_empty() {
            return Err(());
        }
        let mut length = Fraction::from(0);
        for token in text.split(" ") {
            length += match Fraction::from_str(token) {
                Ok(value) => value,
                Err(_) => return Err(()),
            };
        }
        Ok(length)
    }

    fn parse_price(text: &str) -> Result<f64, ()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(0.0);
        }
        match text.parse() {
            Ok(price) => Ok(price),
            Err(_) => Err(()),
        }
    }

    fn setup_callbacks(&self) {
        for field in [
            &self.imp().material_field,
            &self.imp().price_field,
            &self.imp().length_field,
        ] {
            field.connect_changed(clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.validate_fields();
                }
            ));
        }
        self.imp().update_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.update_supply();
            }
        ));
        self.imp().add_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.add_supply();
            }
        ));
        self.imp().delete_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.delete_supply();
                pane.update_fields();
            }
        ));
        self.selection_model().connect_selection_changed(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, _, _| {
                pane.update_fields();
            }
        ));
        self.supplies().connect_items_changed(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, _, _, _| {
                pane.update_content_stack();
            }
        ));
    }

    fn setup_column(&self, field_type: FieldType, property: &'static str, column_title: &str) {
        let factory = gtk::SignalListItemFactory::new();

        // Called when a new row of widgets is added
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::builder().halign(gtk::Align::Start).build();
            list_item.set_child(Some(&label));
        });

        // Called when an object in the model is bound to a row
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let supply_gobject = list_item.item().and_downcast::<SupplyGObject>().unwrap();
            let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
            let value = match field_type {
                FieldType::String => supply_gobject.property::<String>(property),
                FieldType::F64 => supply_gobject.property::<f64>(property).to_string(),
                FieldType::U32 => supply_gobject.property::<u32>(property).to_string(),
                FieldType::Fraction => {
                    Self::format_fraction(&supply_gobject.property::<String>(property))
                }
            };
            label.set_label(&value);
        });

        // Create a column and add it to the view
        self.imp().supplies_view.append_column(
            &gtk::ColumnViewColumn::builder()
                .title(column_title)
                .expand(true)
                .factory(&factory)
                .build(),
        );
    }

    fn setup_column_view(&self) {
        // Create the list store (swap into the RefCell)
        let model = Some(ListStore::new::<SupplyGObject>());
        self.imp().supplies.replace(model);

        // Link the model to the column view
        let selection = gtk::SingleSelection::new(Some(self.supplies()));
        self.imp().supplies_view.set_model(Some(&selection));

        // Add columns to the view and create factories for each
        self.setup_column(FieldType::String, "name", "Name");
        self.setup_column(FieldType::String, "material", "Material");
        self.setup_column(FieldType::F64, "price", "Price");
        self.setup_column(FieldType::U32, "max-quantity", "Quantity");
        self.setup_column(FieldType::String, "length-unit", "Unit");
        self.setup_column(FieldType::Fraction, "length", "Length");
    }

    fn selection_model(&self) -> gtk::SelectionModel {
        self.imp().supplies_view.model().unwrap()
    }

    fn supplies(&self) -> ListStore {
        self.imp().supplies.borrow().clone().unwrap()
    }

    fn update_fields(&self) {
        let selection = self.selection_model().selection();
        if !selection.is_empty() {
            let list_item = self.supplies().item(selection.minimum()).unwrap();
            let supply_gobject = list_item.downcast_ref::<SupplyGObject>().unwrap();

            // Set entry fields based on column view values
            // TODO: Manual match is bad
            let length_unit = match supply_gobject.length_unit().as_str() {
                "Inches" => 0,
                "Centimeters" => 1,
                "Meters" => 2,
                _ => panic!(),
            };
            let imp = self.imp();
            imp.name_field.set_text(&supply_gobject.name());
            imp.material_field.set_text(&supply_gobject.material());
            imp.price_field
                .set_text(&supply_gobject.price().to_string());
            imp.max_quantity_field
                .set_value(supply_gobject.max_quantity() as f64);
            imp.length_unit_field.set_selected(length_unit);
            imp.length_field.set_text(&supply_gobject.length());
        }
    }

    fn update_supply(&self) {
        let selection_model = self.selection_model();
        let selection = selection_model.selection();
        if !selection.is_empty() {
            let i = selection.minimum();
            self.supplies().remove(i);
            self.supplies().insert(i, &self.parse_all_fields());

            // Select the item we just modified
            selection_model.select_item(i, true);
        }
    }

    fn update_content_stack(&self) {
        let content_stack = &self.imp().content_stack;
        if self.supplies().n_items() == 0 {
            content_stack.set_visible_child_name("placeholder");
        } else {
            content_stack.set_visible_child_name("nonempty");
        }
    }

    fn validate_fields(&self) {
        let mut all_valid = true;

        let valid = self.imp().material_field.text_length() != 0;
        Self::highlight_field(&self.imp().material_field, valid);
        all_valid = all_valid && valid;

        let price_field = &self.imp().price_field;
        let valid = Self::parse_price(&price_field.text()).is_ok();
        Self::highlight_field(price_field, valid);
        all_valid = all_valid && valid;

        let length_field = &self.imp().length_field;
        let valid = Self::parse_length(&length_field.text()).is_ok();
        Self::highlight_field(length_field, valid);
        all_valid = all_valid && valid;

        // Disable update and add buttons if any field is invalid
        self.imp().update_button.set_sensitive(all_valid);
        self.imp().add_button.set_sensitive(all_valid);
    }
}
