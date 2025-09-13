use std::cell::{Cell, RefCell};
use std::str::FromStr;

use adw::prelude::*;
use adw::subclass::prelude::*;
use fraction::{Decimal, Fraction, Zero};
use gtk::glib::{Properties, clone, subclass::InitializingObject};
use gtk::{gio::ListStore, glib};

use super::entry::{EntryData, EntryObject};

// TOOD: Optimize creation of new String object (as opposed to passing &str refs)
// TODO: Review pub qualifiers (only use where they make sense)

mod imp {
    use super::*;

    // Object holding the state
    #[derive(gtk::CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::EntryPane)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/entry_pane.ui")]
    pub struct EntryPane {
        // Entry fields
        #[template_child]
        pub name_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub material_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub price_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub quantity_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub length_unit_field: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub length_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub sublength_field: TemplateChild<adw::EntryRow>,

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
        pub column_view: TemplateChild<gtk::ColumnView>,

        // Whether to enable the price field
        #[property(get, set)]
        pub allow_price: Cell<bool>,

        // Whether to require a non-empty value for the quantity
        #[property(get, set)]
        pub require_quantity: Cell<bool>,

        // Data model
        pub entries: RefCell<Option<ListStore>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for EntryPane {
        const NAME: &'static str = "ChopChopEntryPane";
        type Type = super::EntryPane;
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
    impl ObjectImpl for EntryPane {
        // Called when the object is constructed
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_column_view();
            self.obj().setup_callbacks();
            self.obj().update_content_stack();
            self.obj().validate_fields();
            self.obj()
                .bind_property("allow-price", &self.price_field.get(), "visible")
                .build();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for EntryPane {}

    // Trait shared by GTK boxes
    impl BoxImpl for EntryPane {}
}

glib::wrapper! {
    pub struct EntryPane(ObjectSubclass<imp::EntryPane>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl EntryPane {
    pub fn entry_data_vec(&self) -> Vec<EntryData> {
        self.entries()
            .iter::<EntryObject>()
            .filter_map(Result::ok)
            .map(|x| x.entry_data())
            .collect()
    }

    fn add_entry(&self) {
        self.entries().append(&self.parse_all_fields());

        // Select the item we just added
        let selection_model = self.selection_model();
        selection_model.select_item(selection_model.n_items() - 1, true);
    }

    fn delete_entry(&self) {
        let selection = self.selection_model().selection();
        if !selection.is_empty() {
            self.entries().remove(selection.minimum());
        }
    }

    /// Panics if parse_length(text) returns an error
    fn format_fraction(&self, text: &str) -> String {
        // TODO: Make format configurable (original format, fraction, mixed, or decimal)
        // https://docs.rs/fraction/latest/fraction/#format-convert-to-string
        if text.is_empty() {
            String::from("")
        } else {
            format!("{:.10}", self.parse_length(text).unwrap())
        }
    }

    fn format_length(&self, entry_object: &EntryObject) -> String {
        let mut output = self.format_fraction(&entry_object.length());
        // TODO: Use an enum for length units
        output += match entry_object.length_unit() {
            0 => "ft",
            1 => "in",
            2 => "m",
            3 => "cm",
            _ => panic!(),
        };
        if entry_object.length_unit() == 0 {
            output += &format!(" {}in", &self.format_fraction(&entry_object.sublength()));
        }
        output
    }

    fn format_material(&self, entry_object: &EntryObject) -> String {
        entry_object.material().to_string()
    }

    fn format_name(&self, entry_object: &EntryObject) -> String {
        entry_object.name().to_string()
    }

    fn format_price(&self, entry_object: &EntryObject) -> String {
        let price = self.parse_price(&entry_object.price()).unwrap();
        if price == Decimal::zero() {
            String::from("Free")
        } else {
            format!("${:.2}", price)
        }
    }

    fn format_quantity(&self, entry_object: &EntryObject) -> String {
        let quantity = self.parse_quantity(&entry_object.quantity()).unwrap();
        if quantity == -1 {
            String::from("Unlimited")
        } else {
            quantity.to_string()
        }
    }

    fn highlight_field(&self, field: &adw::EntryRow, is_valid: bool) {
        if is_valid {
            field.remove_css_class("invalid-entry");
        } else {
            field.add_css_class("invalid-entry");
        }
    }

    fn parse_all_fields(&self) -> EntryObject {
        EntryObject::new(
            self.imp().name_field.text().to_string(),
            self.imp().material_field.text().to_string(),
            self.imp().price_field.text().to_string(),
            self.imp().quantity_field.text().to_string(),
            self.imp().length_unit_field.selected(),
            self.imp().length_field.text().to_string(),
            self.imp().sublength_field.text().to_string(),
        )
    }

    // TODO: Move parsing logic somewhere else?

    pub fn parse_length(&self, text: &str) -> Result<Fraction, ()> {
        let tokens: Vec<_> = text.trim().split(" ").filter(|s| !s.is_empty()).collect();
        if tokens.is_empty() || (tokens.len() > 2) {
            Err(())
        } else {
            let mut length = Fraction::zero();
            for token in tokens {
                length += match Fraction::from_str(token) {
                    Ok(value) if (value >= Fraction::zero()) => value,
                    _ => return Err(()),
                };
            }
            Ok(length)
        }
    }

    // TODO: Consolidate
    pub fn parse_length_optional(&self, text: &str) -> Result<Fraction, ()> {
        let tokens: Vec<_> = text.trim().split(" ").filter(|s| !s.is_empty()).collect();
        if tokens.len() > 2 {
            Err(())
        } else {
            let mut length = Fraction::zero();
            for token in tokens {
                length += match Fraction::from_str(token) {
                    Ok(value) if (value >= Fraction::zero()) => value,
                    _ => return Err(()),
                };
            }
            Ok(length)
        }
    }

    /// Currently this allows specifying price via fraction string
    pub fn parse_price(&self, text: &str) -> Result<Decimal, ()> {
        let text = text.trim();
        if text.is_empty() {
            Ok(Decimal::zero())
        } else {
            match Decimal::from_str(text) {
                Ok(value) if (value >= Decimal::zero()) => Ok(value),
                _ => Err(()),
            }
        }
    }

    pub fn parse_quantity(&self, text: &str) -> Result<i64, ()> {
        let text = text.trim();
        if text.is_empty() {
            if self.require_quantity() {
                Err(())
            } else {
                // Unlimited
                Ok(-1)
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) if (value >= 0) => Ok(value),
                _ => Err(()),
            }
        }
    }

    fn selection_model(&self) -> gtk::SelectionModel {
        self.imp().column_view.model().unwrap()
    }

    fn setup_callbacks(&self) {
        for field in [
            &self.imp().material_field,
            &self.imp().price_field,
            &self.imp().quantity_field,
            &self.imp().length_field,
            &self.imp().sublength_field,
        ] {
            field.connect_changed(clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.validate_fields();
                }
            ));
        }
        self.imp().length_unit_field.connect_selected_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.update_length_unit();
                pane.validate_fields();
            }
        ));
        self.imp().update_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.update_entry();
            }
        ));
        self.imp().add_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.add_entry();
            }
        ));
        self.imp().delete_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.delete_entry();
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
        self.entries().connect_items_changed(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, _, _, _| {
                pane.update_content_stack();
            }
        ));
        self.connect_require_quantity_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.validate_fields();
            }
        ));
    }

    fn setup_column(
        &self,
        format_column_fn: fn(&Self, &EntryObject) -> String,
        column_title: &str,
    ) -> gtk::ColumnViewColumn {
        let factory = gtk::SignalListItemFactory::new();

        // Called when a new row of widgets is added
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::builder().halign(gtk::Align::Start).build();
            list_item.set_child(Some(&label));
        });

        // Called when an object in the model is bound to a row
        factory.connect_bind(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let entry_object = list_item.item().and_downcast::<EntryObject>().unwrap();
                let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
                label.set_label(&format_column_fn(&pane, &entry_object));
            }
        ));

        // Create a column and add it to the view
        let column = gtk::ColumnViewColumn::builder()
            .title(column_title)
            .expand(true)
            .factory(&factory)
            .build();
        self.imp().column_view.append_column(&column);
        column
    }

    fn setup_column_view(&self) {
        // Create the list store (swap into the RefCell)
        let model = Some(ListStore::new::<EntryObject>());
        self.imp().entries.replace(model);

        // Link the model to the column view
        let selection = gtk::SingleSelection::new(Some(self.entries()));
        self.imp().column_view.set_model(Some(&selection));

        // Add columns to the view and create factories for each
        self.setup_column(Self::format_name, "Name");
        self.setup_column(Self::format_material, "Material");
        let price_column = self.setup_column(Self::format_price, "Price");
        self.setup_column(Self::format_quantity, "Quantity");
        self.setup_column(Self::format_length, "Length");

        // Visibility of the price column is determined by the allow-price property
        self.bind_property("allow-price", &price_column, "visible")
            .build();
    }

    fn entries(&self) -> ListStore {
        self.imp().entries.borrow().clone().unwrap()
    }

    fn update_content_stack(&self) {
        let content_stack = &self.imp().content_stack;
        if self.entries().n_items() == 0 {
            content_stack.set_visible_child_name("placeholder");
        } else {
            content_stack.set_visible_child_name("nonempty");
        }
    }

    fn update_fields(&self) {
        let selection = self.selection_model().selection();
        if !selection.is_empty() {
            let list_item = self.entries().item(selection.minimum()).unwrap();
            let entry_object = list_item.downcast_ref::<EntryObject>().unwrap();

            // Set entry fields based on column view values
            let imp = self.imp();
            imp.name_field.set_text(&entry_object.name());
            imp.material_field.set_text(&entry_object.material());
            imp.price_field.set_text(&entry_object.price());
            imp.quantity_field.set_text(&entry_object.quantity());
            imp.length_field.set_text(&entry_object.length());
            imp.sublength_field.set_text(&entry_object.sublength());

            // Do this after setting sublength_field to skip the field entry animation
            imp.length_unit_field
                .set_selected(entry_object.length_unit());
        }
    }

    fn update_length_unit(&self) {
        let sublength_field = &self.imp().sublength_field;
        if self.use_sublength() {
            sublength_field.set_visible(true);
        } else {
            sublength_field.set_visible(false);
            sublength_field.set_text("");
        }
        let selection = self.imp().length_unit_field.selected();
        self.imp().length_field.set_title(match selection {
            0 => "Feet",
            _ => "Length",
        });
    }

    fn update_entry(&self) {
        let selection_model = self.selection_model();
        let selection = selection_model.selection();
        if !selection.is_empty() {
            let i = selection.minimum();
            self.entries().remove(i);
            self.entries().insert(i, &self.parse_all_fields());

            // Select the item we just modified
            selection_model.select_item(i, true);
        }
    }

    fn use_sublength(&self) -> bool {
        self.imp().length_unit_field.selected() == 0
    }

    fn validate_fields(&self) {
        let mut all_valid = true;

        let valid = self.imp().material_field.text_length() != 0;
        self.highlight_field(&self.imp().material_field, valid);
        all_valid = all_valid && valid;

        let price_field = &self.imp().price_field;
        let valid = self.parse_price(&price_field.text()).is_ok();
        self.highlight_field(price_field, valid);
        all_valid = all_valid && valid;

        let quantity_field = &self.imp().quantity_field;
        let valid = self.parse_quantity(&quantity_field.text()).is_ok();
        self.highlight_field(quantity_field, valid);
        all_valid = all_valid && valid;

        let length_field = &self.imp().length_field;
        let valid = self.parse_length(&length_field.text()).is_ok();
        self.highlight_field(length_field, valid);
        all_valid = all_valid && valid;

        if self.use_sublength() {
            let sublength_field = &self.imp().sublength_field;
            let valid = self.parse_length(&sublength_field.text()).is_ok();
            self.highlight_field(sublength_field, valid);
            all_valid = all_valid && valid;
        }

        // Disable update and add buttons if any field is invalid
        self.imp().update_button.set_sensitive(all_valid);
        self.imp().add_button.set_sensitive(all_valid);
    }
}
