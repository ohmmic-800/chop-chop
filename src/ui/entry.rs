use gtk::glib;
use gtk::glib::{Object, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

/// Represents the user-entered data for a supply row
///
/// Stores user-entered strings instead of parsed numeric types (fraction::Decimal or
/// fraction::Fraction). This allows editing the original text when a row is re-selected, and avoids
/// headaches with storing custom types in a GObject.
#[derive(Default)]
pub struct EntryData {
    pub name: String,
    pub material: String,
    pub price: String,
    pub quantity: String,
    pub length_unit: u32,
    pub length: String,
    pub sublength: String,
}

// Wrap SupplyData in a GObject so it can be used in a gtk::ListStore
// https://gtk-rs.org/gtk4-rs/git/book/list_widgets.html#views
mod imp {
    use super::*;

    // Object holding the state
    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::EntryObject)]
    pub struct EntryObject {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "material", get, set, type = String, member = material)]
        #[property(name = "price", get, set, type = String, member = price)]
        #[property(name = "quantity", get, set, type = String, member = quantity)]
        #[property(name = "length-unit", get, set, type = u32, member = length_unit)]
        #[property(name = "length", get, set, type = String, member = length)]
        #[property(name = "sublength", get, set, type = String, member = sublength)]
        pub data: RefCell<EntryData>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for EntryObject {
        const NAME: &'static str = "ChopChopEntryObject";
        type Type = super::EntryObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for EntryObject {}
}

glib::wrapper! {
    pub struct EntryObject(ObjectSubclass<imp::EntryObject>);
}

impl EntryObject {
    pub fn new(
        name: String,
        material: String,
        price: String,
        quantity: String,
        length_unit: u32,
        length: String,
        sublength: String,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("material", material)
            .property("price", price)
            .property("quantity", quantity)
            .property("length_unit", length_unit)
            .property("length", length)
            .property("sublength", sublength)
            .build()
    }
}
