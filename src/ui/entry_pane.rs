use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{Properties, clone, subclass::InitializingObject};
use gtk::{gio::ListStore, glib};

use super::entry::{EntryData, EntryObject};
use super::utils::*;
use crate::units::LengthUnit;

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

        // Whether all entry fields are valid
        #[property(get, set)]
        pub all_entries_valid: Cell<bool>,

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
            let obj = self.obj();
            obj.setup_column_view();
            obj.setup_callbacks();
            obj.setup_bindings();
            obj.update_placeholder();
            obj.validate_all_entries();
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
    pub fn entry_data(&self) -> Vec<EntryData> {
        self.entries()
            .iter::<EntryObject>()
            .filter_map(Result::ok)
            .map(|x| x.entry_data())
            .collect()
    }

    fn add_entry(&self) {
        self.entries().append(&self.create_entry_object());
        self.selection_model().select_item(self.n_items() - 1, true);
    }

    fn column(&self, column_index: u32) -> gtk::ColumnViewColumn {
        self.imp()
            .column_view
            .columns()
            .item(column_index)
            .and_downcast::<gtk::ColumnViewColumn>()
            .unwrap()
    }

    fn delete_entry(&self) {
        if self.n_items() != 0 {
            self.entries().remove(self.selection());
        }
    }

    fn create_entry_object(&self) -> EntryObject {
        let imp = self.imp();
        let sublength = if self.use_sublength() {
            imp.sublength_field.text().to_string()
        } else {
            String::new()
        };
        EntryObject::new(
            imp.name_field.text().to_string(),
            imp.material_field.text().to_string(),
            imp.price_field.text().to_string(),
            imp.quantity_field.text().to_string(),
            imp.length_unit_field.selected(),
            imp.length_field.text().to_string(),
            sublength,
        )
    }

    fn entries(&self) -> ListStore {
        self.imp().entries.borrow().clone().unwrap()
    }

    pub fn main_unit_title(unit: LengthUnit) -> &'static str {
        match unit {
            LengthUnit::FeetInches => "Feet",
            _ => "Length",
        }
    }

    fn n_items(&self) -> u32 {
        self.entries().n_items()
    }

    fn selection(&self) -> u32 {
        self.selection_model().selection().minimum()
    }

    fn selection_model(&self) -> gtk::SelectionModel {
        self.imp().column_view.model().unwrap()
    }

    fn setup_bindings(&self) {
        let imp = self.imp();

        // Show the sublength field only if applicable
        imp.length_unit_field
            .bind_property("selected", &imp.sublength_field.get(), "visible")
            .transform_to(|_, i| Some(parse_length_unit(i).has_subunit()))
            .build();

        // Set the title of the length field based on the unit type
        imp.length_unit_field
            .bind_property("selected", &imp.length_field.get(), "title")
            .transform_to(|_, i| Some(Self::main_unit_title(parse_length_unit(i))))
            .build();

        self.bind_property("allow-price", &imp.price_field.get(), "visible")
            .build();
        self.bind_property("allow-price", &self.column(2), "visible")
            .build();
        for button in [&imp.update_button.get(), &imp.add_button.get()] {
            self.bind_property("all-entries-valid", button, "sensitive")
                .build();
        }
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();
        for field in [
            &imp.material_field,
            &imp.price_field,
            &imp.quantity_field,
            &imp.length_field,
            &imp.sublength_field,
        ] {
            field.connect_changed(clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.validate_all_entries();
                }
            ));
        }
        imp.length_unit_field.connect_selected_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.validate_all_entries();
            }
        ));
        imp.update_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.update_entry();
            }
        ));
        imp.add_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.add_entry();
            }
        ));
        imp.delete_button.connect_clicked(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.delete_entry();
                pane.update_fields();
            }
        ));
        self.entries().connect_items_changed(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, _, _, _| {
                pane.update_placeholder();
            }
        ));
        self.selection_model().connect_selection_changed(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_, _, _| {
                pane.update_fields();
            }
        ));
        self.connect_require_quantity_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |_| {
                pane.validate_all_entries();
            }
        ));
    }

    fn setup_column_factory<F>(&self, column_index: u32, format_column: F)
    where
        F: Fn(&EntryObject) -> String + 'static,
    {
        let factory = gtk::SignalListItemFactory::new();
        self.column(column_index).set_factory(Some(&factory));

        // Called when widgets need to be created for a new row
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::builder().halign(gtk::Align::Start).build();
            list_item.set_child(Some(&label));
        });

        // Called when a list item is bound to a row
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry_object = list_item.item().and_downcast::<EntryObject>().unwrap();
            let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_label(&format_column(&entry_object));
        });
    }

    fn setup_column_view(&self) {
        // Set up the ListStore model
        let model = Some(ListStore::new::<EntryObject>());
        self.imp().entries.replace(model);
        let selection = gtk::SingleSelection::new(Some(self.entries()));
        self.imp().column_view.set_model(Some(&selection));

        // Define EntryObject -> String mappings for each column
        self.setup_column_factory(0, |e| e.name().to_string());
        self.setup_column_factory(1, |e| e.material().to_string());
        self.setup_column_factory(2, |e| format_price(parse_price(&e.price(), true).unwrap()));
        self.setup_column_factory(3, |e| {
            format_quantity(parse_quantity(&e.quantity(), true).unwrap())
        });
        self.setup_column_factory(4, |e| {
            format_length(
                parse_length(&e.length(), true).unwrap(),
                parse_length(&e.sublength(), true).unwrap(),
                parse_length_unit(e.length_unit()),
            )
        });
    }

    fn update_entry(&self) {
        if self.n_items() != 0 {
            let i = self.selection();
            self.entries().remove(i);
            self.entries().insert(i, &self.create_entry_object());
            self.selection_model().select_item(i, true);
        }
    }

    fn update_fields(&self) {
        if self.n_items() != 0 {
            let list_item = self.entries().item(self.selection());
            let entry_object = list_item.and_downcast::<EntryObject>().unwrap();

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

    fn update_placeholder(&self) {
        let name = if self.n_items() == 0 {
            "placeholder"
        } else {
            "nonempty"
        };
        self.imp().content_stack.set_visible_child_name(name);
    }

    fn use_sublength(&self) -> bool {
        parse_length_unit(self.imp().length_unit_field.selected()).has_subunit()
    }

    fn validate_all_entries(&self) {
        let mut all_valid = true;
        let imp = self.imp();
        all_valid &= Self::validate_entry(&imp.material_field.get(), |e| e.text_length() != 0);
        all_valid &= Self::validate_entry(&imp.price_field.get(), |e| {
            parse_price(&e.text(), true).is_ok()
        });
        all_valid &= Self::validate_entry(&imp.quantity_field.get(), |e| {
            parse_quantity(&e.text(), !self.require_quantity()).is_ok()
        });
        all_valid &= Self::validate_entry(&imp.length_field.get(), |e| {
            parse_length(&e.text(), false).is_ok()
        });
        if self.use_sublength() {
            all_valid &= Self::validate_entry(&imp.sublength_field.get(), |e| {
                parse_length(&e.text(), true).is_ok()
            });
        }
        self.set_all_entries_valid(all_valid);
    }

    fn validate_entry<F>(entry: &adw::EntryRow, validate: F) -> bool
    where
        F: Fn(&adw::EntryRow) -> bool,
    {
        let is_valid = validate(entry);
        if !is_valid {
            entry.add_css_class("invalid-entry");
        } else {
            entry.remove_css_class("invalid-entry");
        }
        is_valid
    }
}
