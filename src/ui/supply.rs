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
    pub price: f32,
    pub max_quantity: u32,
    pub length_unit: String,
    pub length: f32,
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
        #[property(name = "price", get, set, type = f32, member = price)]
        #[property(name = "max-quantity", get, set, type = u32, member = max_quantity)]
        #[property(name = "length-unit", get, set, type = String, member = length_unit)]
        #[property(name = "length", get, set, type = f32, member = length)]
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
        price: f32,
        max_quantity: u32,
        length_unit: String,
        length: f32,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("material", material)
            .property("price", price)
            .property("max-quantity", max_quantity)
            .property("length_unit", length_unit)
            .property("length", length)
            .build()
    }
}
