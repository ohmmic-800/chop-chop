use gtk::glib;
use gtk::glib::{Object, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

/// Represents the user-entered data for a part row
///
/// Units need to be normalized before converting to crate::modeling::Part
#[derive(Default)]
pub struct PartData {
    pub name: String,
    pub material: String,
    pub max_quantity: u32,
    pub length_unit: String,
    pub length: f32,
}

// Wrap PartData in a GObject so it can be used in a gtk::ListStore
// https://gtk-rs.org/gtk4-rs/git/book/list_widgets.html#views
mod imp {
    use super::*;

    // Object holding the state
    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::PartGObject)]
    pub struct PartGObject {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "material", get, set, type = String, member = material)]
        #[property(name = "max-quantity", get, set, type = u32, member = max_quantity)]
        #[property(name = "length-unit", get, set, type = String, member = length_unit)]
        #[property(name = "length", get, set, type = f32, member = length)]
        pub data: RefCell<PartData>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PartGObject {
        const NAME: &'static str = "ChopChopPartGObject";
        type Type = super::PartGObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for PartGObject {}
}

glib::wrapper! {
    pub struct PartGObject(ObjectSubclass<imp::PartGObject>);
}

impl PartGObject {
    pub fn new(
        name: String,
        material: String,
        max_quantity: u32,
        length_unit: String,
        length: f32,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("material", material)
            .property("max-quantity", max_quantity)
            .property("length_unit", length_unit)
            .property("length", length)
            .build()
    }
}
