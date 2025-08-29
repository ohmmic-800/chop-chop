use glib::Object;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

// TODO: Consolidate data model here with that in modeling.rs

mod imp {
    use super::*;

    // Object holding the state
    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::MaterialObject)]
    pub struct MaterialsObject {
        #[property(name = "description", get, set, type = String, member = description)]
        #[property(name = "quantity", get, set, type = u32, member = quantity)]
        #[property(name = "price", get, set, type = f32, member = price)]
        #[property(name = "length-unit", get, set, type = String, member = length_unit)]
        #[property(name = "length", get, set, type = f32, member = length)]
        pub data: RefCell<MaterialsData>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for MaterialsObject {
        const NAME: &'static str = "ChopChopMaterialsObject";
        type Type = super::MaterialObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for MaterialsObject {}
}

#[derive(Default)]
pub struct MaterialsData {
    pub description: String,
    pub quantity: u32,
    pub price: f32,
    pub length_unit: String,
    pub length: f32,
}

glib::wrapper! {
    pub struct MaterialObject(ObjectSubclass<imp::MaterialsObject>);
}

impl MaterialObject {
    pub fn new(
        description: String,
        quantity: u32,
        price: f32,
        length_unit: String,
        length: f32,
    ) -> Self {
        Object::builder()
            .property("description", description)
            .property("quantity", quantity)
            .property("price", price)
            .property("length-unit", length_unit)
            .property("length", length)
            .build()
    }
}
