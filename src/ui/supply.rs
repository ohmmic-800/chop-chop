use gtk::glib;
use gtk::glib::{Object, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

/// Represents the user-entered data for a supply row
///
/// Units need to be normalized before converting to crate::modeling::Supply
#[derive(Default)]
pub struct SupplyData {
    pub name: String,
    pub material: String,
    pub price: f64,
    pub max_quantity: u32,
    pub length_unit: u32,

    // Store the user-entered string instead of a parsed fraction::Fraction. This allows subsequent
    // edits in the original format and avoids headaches with storing custom types in a GObject.
    pub length: String,
    pub sublength: String,
}

// Wrap SupplyData in a GObject so it can be used in a gtk::ListStore
// https://gtk-rs.org/gtk4-rs/git/book/list_widgets.html#views
mod imp {
    use super::*;

    // Object holding the state
    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::SupplyGObject)]
    pub struct SupplyGObject {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "material", get, set, type = String, member = material)]
        #[property(name = "price", get, set, type = f64, member = price)]
        #[property(name = "max-quantity", get, set, type = u32, member = max_quantity)]
        #[property(name = "length-unit", get, set, type = u32, member = length_unit)]
        #[property(name = "length", get, set, type = String, member = length)]
        #[property(name = "sublength", get, set, type = String, member = sublength)]
        pub data: RefCell<SupplyData>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SupplyGObject {
        const NAME: &'static str = "ChopChopSupplyGObject";
        type Type = super::SupplyGObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for SupplyGObject {}
}

glib::wrapper! {
    pub struct SupplyGObject(ObjectSubclass<imp::SupplyGObject>);
}

impl SupplyGObject {
    pub fn new(
        name: String,
        material: String,
        price: f64,
        max_quantity: u32,
        length_unit: u32,
        length: String,
        sublength: String,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("material", material)
            .property("price", price)
            .property("max-quantity", max_quantity)
            .property("length_unit", length_unit)
            .property("length", length)
            .property("sublength", sublength)
            .build()
    }
}
