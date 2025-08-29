use crate::ui::materials_object::MaterialObject;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use glib::{Object, clone};
use gtk::{CompositeTemplate, gio, glib, prelude::ButtonExt};
use std::cell::RefCell;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/window.ui")]
    pub struct Window {
        #[template_child]
        pub description_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub quantity_spin: TemplateChild<adw::SpinRow>,

        #[template_child]
        pub price_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub length_unit_combo: TemplateChild<adw::ComboRow>,

        #[template_child]
        pub length_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub materials_list: TemplateChild<gtk::ListBox>,

        pub materials: RefCell<Option<gio::ListStore>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "ChopChopWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for Window {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_materials();
            obj.setup_callbacks();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for Window {}

    // Trait shared by all windows
    impl WindowImpl for Window {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for Window {}

    // Trait shared by all Adwaita application windows
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn materials(&self) -> gio::ListStore {
        self.imp()
            .materials
            .borrow()
            .clone()
            .expect("Could not get current materials.")
    }

    fn setup_materials(&self) {
        // Create new model
        let model = gio::ListStore::new::<MaterialObject>();

        // Set model
        self.imp().materials.replace(Some(model.clone()));

        // Wrap model selection and pass it to the list box
        let selection_model = gtk::NoSelection::new(Some(self.materials()));
        self.imp().materials_list.bind_model(
            Some(&selection_model),
            clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or_panic]
                move |obj| {
                    let material_object = obj
                        .downcast_ref()
                        .expect("The object should be of type `MaterialObject`.");
                    let row = window.create_material_row(material_object);
                    row.upcast()
                }
            ),
        );
    }

    fn create_material_row(&self, material_object: &MaterialObject) -> adw::ActionRow {
        let row = adw::ActionRow::builder().build();
        let row_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();
        row.add_prefix(&row_box);
        row_box.append(&gtk::Label::new(Some(&material_object.description())));
        row_box.append(&gtk::Label::new(Some(
            &material_object.quantity().to_string(),
        )));
        row_box.append(&gtk::Label::new(Some(&material_object.price().to_string())));
        row_box.append(&gtk::Label::new(Some(
            &material_object.length().to_string(),
        )));
        row_box.append(&gtk::Label::new(Some(
            &material_object.length_unit().to_string(),
        )));
        row
    }

    fn setup_callbacks(&self) {
        // Setup callback for clicking the add button
        self.imp().add_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.new_material();
            }
        ));
    }

    fn new_material(&self) {
        let imp = self.imp();
        let material = MaterialObject::new(
            imp.description_entry.text().to_string(),
            imp.quantity_spin.value() as u32,
            imp.price_entry.text().parse().unwrap(),
            // TODO: There has to be a better way to do this...
            String::from(match imp.length_unit_combo.selected() {
                0 => "Inches",
                1 => "Centimeters",
                2 => "Meters",
                _ => panic!(),
            }),
            imp.length_entry.text().parse().unwrap(),
        );
        self.materials().append(&material);
    }
}
