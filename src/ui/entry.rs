use std::cell::{Cell, RefCell};

use gtk::glib;
use gtk::glib::{Object, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use serde::{Deserialize, Serialize};

use super::utils::*;
use crate::size::{FractionFormat, Size};

/// Represents the user-entered data for a supply row
///
/// Stores user-entered strings instead of parsed numeric types (fraction::Decimal or
/// fraction::Fraction). This allows editing the original text when a row is re-selected, and avoids
/// headaches with storing custom types in a GObject.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntryData {
    pub dimension: u32,
    pub name: String,
    pub material: String,
    pub price: String,
    pub quantity: String,
    pub length_unit: u32,
    pub major_length: String,
    pub minor_length: String,
    pub width_unit: u32,
    pub major_width: String,
    pub minor_width: String,
}

// Wrap SupplyData in a GObject so it can be used in a gtk::ListStore
// https://gtk-rs.org/gtk4-rs/git/book/list_widgets.html#views
mod imp {
    use super::*;

    // Object holding the state
    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::EntryObject)]
    pub struct EntryObject {
        // Raw user entries
        // Intended for serde
        // TODO: Can we use &str here to avoid copying when getting property values?
        #[property(name = "dimension", get, set, type = u32, member = dimension)]
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "material", get, set, type = String, member = material)]
        #[property(name = "price", get, set, type = String, member = price)]
        #[property(name = "quantity", get, set, type = String, member = quantity)]
        #[property(name = "length-unit", get, set, type = u32, member = length_unit)]
        #[property(name = "major-length", get, set, type = String, member = major_length)]
        #[property(name = "minor-length", get, set, type = String, member = minor_length)]
        #[property(name = "width-unit", get, set, type = u32, member = width_unit)]
        #[property(name = "major-width", get, set, type = String, member = major_width)]
        #[property(name = "minor-width", get, set, type = String, member = minor_width)]
        pub entry_data: RefCell<EntryData>,

        // Strings for direct binding to display columns
        // Not intended for serde
        #[property(get, set)]
        pub price_display: RefCell<String>,
        #[property(get, set)]
        pub quantity_display: RefCell<String>,
        #[property(get, set)]
        pub length_display: RefCell<String>,
        #[property(get, set)]
        pub width_display: RefCell<String>,

        // Values used for sorting in the column view
        // Not intended for serde
        #[property(get, set)]
        pub price_sort: Cell<f64>,
        #[property(get, set)]
        pub quantity_sort: Cell<i64>,
        #[property(get, set)]
        pub length_sort: Cell<f64>,
        #[property(get, set)]
        pub width_sort: Cell<f64>,

        // Properties that determine formatting
        #[property(get, set)]
        pub price_precision: Cell<u32>,
        #[property(get, set)]
        pub(super) size_format: Cell<u32>,
        #[property(get, set)]
        pub(super) size_precision: Cell<u32>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for EntryObject {
        const NAME: &'static str = "ChopChopEntryObject";
        type Type = super::EntryObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for EntryObject {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_callbacks();
        }
    }
}

glib::wrapper! {
    pub struct EntryObject(ObjectSubclass<imp::EntryObject>);
}

impl EntryObject {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn entry_data(&self) -> EntryData {
        self.imp().entry_data.borrow().clone()
    }

    pub fn replace_data(&self, data: EntryData) {
        self.imp().entry_data.replace(data);
        for property_name in [
            "dimension",
            "name",
            "material",
            "price",
            "quantity",
            "length-unit",
            "major-length",
            "minor-length",
            "width-unit",
            "major-width",
            "minor-width",
        ] {
            self.notify(property_name);
        }
    }

    fn setup_callbacks(&self) {
        // Callbacks for updating display strings
        for property_name in ["price", "price-precision"] {
            self.connect_notify(Some(property_name), |entry_object, _| {
                entry_object.update_price_display();
            });
        }
        self.connect_notify(Some("quantity"), |entry_object, _| {
            entry_object.update_quantity_display();
        });
        for property_name in [
            "length-unit",
            "major-length",
            "minor-length",
            "size-format",
            "size-precision",
        ] {
            self.connect_notify(Some(property_name), |entry_object, _| {
                entry_object.update_length_display();
            });
        }
        for property_name in [
            "width-unit",
            "major-width",
            "minor-width",
            "size-format",
            "size-precision",
        ] {
            self.connect_notify(Some(property_name), |entry_object, _| {
                entry_object.update_width_display();
            });
        }

        // Callbacks for updating sort properties
        self.connect_notify(Some("price"), |entry_object, _| {
            entry_object.update_price_sort();
        });
        self.connect_notify(Some("quantity"), |entry_object, _| {
            entry_object.update_quantity_sort();
        });
        for property_name in ["length-unit", "major-length", "minor-length"] {
            self.connect_notify(Some(property_name), |entry_object, _| {
                entry_object.update_length_sort();
            });
        }
        for property_name in ["width-unit", "major-width", "minor-width"] {
            self.connect_notify(Some(property_name), |entry_object, _| {
                entry_object.update_width_sort();
            });
        }
    }

    fn update_price_display(&self) {
        let price = parse_price(&self.price(), true).unwrap();
        self.set_price_display(format_price(price, self.price_precision()));
    }

    fn update_quantity_display(&self) {
        let quantity = parse_quantity(&self.quantity(), true).unwrap();
        self.set_quantity_display(format_quantity(quantity));
    }

    fn update_length_display(&self) {
        let length = Size::from(
            self.length_unit(),
            &self.major_length(),
            &self.minor_length(),
        );
        let format = FractionFormat::from(self.size_format(), self.size_precision());
        self.set_length_display(length.format(&format));
    }

    fn update_width_display(&self) {
        if self.dimension() == 1 {
            let width = Size::from(self.width_unit(), &self.major_width(), &self.minor_width());
            let format = FractionFormat::from(self.size_format(), self.size_precision());
            self.set_width_display(width.format(&format));
        } else {
            self.set_width_display("");
        }
    }

    fn update_price_sort(&self) {
        let price = parse_price(&self.price(), true).unwrap();
        let price_sort: f64 = price.try_into().unwrap();
        self.set_price_sort(price_sort);
    }

    fn update_quantity_sort(&self) {
        let quantity = parse_quantity(&self.quantity(), true).unwrap();
        if quantity >= 0 {
            self.set_quantity_sort(quantity);
        } else {
            self.set_quantity_sort(i64::MAX);
        }
    }

    fn update_length_sort(&self) {
        let length = Size::from(
            self.length_unit(),
            &self.major_length(),
            &self.minor_length(),
        );
        let length_sort: f64 = length.to_meters().try_into().unwrap();
        self.set_length_sort(length_sort);
    }

    fn update_width_sort(&self) {
        let width = Size::from(self.width_unit(), &self.major_width(), &self.minor_width());
        let width_sort: f64 = width.to_meters().try_into().unwrap();
        self.set_width_sort(width_sort);
    }
}
