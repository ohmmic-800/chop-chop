use std::cell::{Cell, RefCell};
use std::sync::OnceLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gdk::{Key, ModifierType};
use gtk::glib::{Properties, clone, subclass::InitializingObject, subclass::Signal};
use gtk::{CompositeTemplate, gio::ListStore, glib};

use super::entry::{EntryData, EntryObject};
use super::unit_row::UnitRow;
use super::utils::*;
use crate::size::SizeUnit;

mod imp {
    use super::*;

    // Object holding the state
    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::EntryPane)]
    #[template(resource = "/com/ohmm-software/Chop-Chop/entry_pane.ui")]
    pub struct EntryPane {
        // Entry fields
        #[template_child]
        pub(super) dimension_field: TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub(super) name_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) material_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) price_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) quantity_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) length_unit_field: TemplateChild<UnitRow>,
        #[template_child]
        pub(super) major_length_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) minor_length_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) width_unit_field: TemplateChild<UnitRow>,
        #[template_child]
        pub(super) major_width_field: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) minor_width_field: TemplateChild<adw::EntryRow>,

        // Used to change the title of the property entry fields
        #[template_child]
        pub(super) properties_group: TemplateChild<adw::PreferencesGroup>,

        // Used to toggle visibility of width entry fields
        #[template_child]
        pub(super) width_group: TemplateChild<adw::PreferencesGroup>,

        // Used to switch between the column view and a placeholder
        #[template_child]
        pub(super) content_stack: TemplateChild<gtk::Stack>,

        // Column view
        #[template_child]
        pub(super) column_view: TemplateChild<gtk::ColumnView>,

        // A word for describing entries ("Supply" or "Part")
        #[property(get, set)]
        pub(super) entry_descriptor: RefCell<String>,

        // Whether to enable the price field
        #[property(get, set)]
        pub(super) allow_price: Cell<bool>,

        // Whether to require a non-empty value for the quantity
        #[property(get, set)]
        pub(super) require_quantity: Cell<bool>,

        // Whether all entry fields are valid
        #[property(get, set)]
        pub(super) all_entries_valid: Cell<bool>,

        // User preferences (should be bound to settings values)
        #[property(get, set)]
        pub(super) size_format: Cell<u32>,
        #[property(get, set)]
        pub(super) size_precision: Cell<u32>,
        #[property(get, set)]
        pub(super) price_precision: Cell<u32>,
        #[property(get, set)]
        pub(super) default_unit: Cell<u32>,
        #[property(get, set)]
        pub(super) deselect_add: Cell<bool>,
        #[property(get, set)]
        pub(super) default_material_1d: RefCell<String>,
        #[property(get, set)]
        pub(super) default_material_2d: RefCell<String>,

        // Data model
        pub(super) entries: RefCell<Option<ListStore>>,
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

            // Set up widget actions
            klass.install_action("pane.add", None, |pane, _, _| pane.add_entry());
            klass.install_action("pane.delete", None, |pane, _, _| pane.delete_entry());
            klass.install_action("pane.next", None, |pane, _, _| pane.next_row());
            klass.install_action("pane.prev", None, |pane, _, _| pane.prev_row());
            klass.install_action("pane.unsort", None, |pane, _, _| pane.unsort());

            // Set up keybindings for widget actions
            klass.add_binding_action(Key::Return, ModifierType::CONTROL_MASK, "pane.add");
            klass.add_binding_action(Key::D, ModifierType::CONTROL_MASK, "pane.delete");
            klass.add_binding_action(Key::J, ModifierType::CONTROL_MASK, "pane.next");
            klass.add_binding_action(Key::K, ModifierType::CONTROL_MASK, "pane.prev");
            klass.add_binding_action(Key::U, ModifierType::CONTROL_MASK, "pane.unsort");
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
            obj.setup_entries();
            obj.setup_column_view();
            obj.setup_callbacks();
            obj.setup_bindings();
            obj.update_fields();
            obj.update_placeholder();
            obj.validate_all_entries();
            obj.update_can_delete();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            // The parent window can watch this signal to track unsaved changes
            SIGNALS.get_or_init(|| vec![Signal::builder("entries-updated").build()])
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

    pub fn replace_entry_data(&self, entry_data: Vec<EntryData>) {
        let entries = self.entries();
        entries.remove_all();
        for item in entry_data {
            let entry_object = self.new_entry_object();
            entry_object.replace_data(item);
            entries.append(&entry_object);
        }
    }

    fn add_entry(&self) {
        let entry_object = self.new_entry_object();
        self.entries().append(&entry_object);
        self.update_entry_object(&entry_object);
        if self.deselect_add() {
            self.selection_model().unselect_all();
        } else {
            // TODO: Looping is not optimal - we could avoid this if we knew the sort permutation
            // Maybe unsort -> add -> sort?
            for j in 0..self.n_items() {
                if entry_object == self.selection_model_item(j) {
                    self.selection_model().select_item(j, true);
                    break;
                }
            }
        }
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
        if let Some(i) = self.selection() {
            // TODO: Looping is not optimal - we could avoid this if we knew the sort permutation
            // Maybe unsort -> delete -> sort?
            let entry_object = self.selection_model_item(i);
            let entries = self.entries();
            for j in 0..self.n_items() {
                if entry_object == entries.item(j).and_downcast::<EntryObject>().unwrap() {
                    entries.remove(j);
                    break;
                }
            }

            if self.n_items() != 0 {
                let j = if i >= self.n_items() {
                    self.n_items() - 1
                } else {
                    i
                };
                self.selection_model().select_item(j, true);
            }
        }
        self.update_can_delete();
        self.signal_entries_updated();
        self.update_fields();
    }

    fn entries(&self) -> ListStore {
        self.imp().entries.borrow().clone().unwrap()
    }

    fn n_items(&self) -> u32 {
        self.entries().n_items()
    }

    fn new_entry_object(&self) -> EntryObject {
        let entry_object = EntryObject::new();
        self.bind_property("size-format", &entry_object, "size-format")
            .sync_create()
            .build();
        self.bind_property("size-precision", &entry_object, "size-precision")
            .sync_create()
            .build();
        self.bind_property("price-precision", &entry_object, "price-precision")
            .sync_create()
            .build();
        entry_object
    }

    fn next_row(&self) {
        match self.selection() {
            None => self.set_selection(Some(0)),
            Some(i) if i == self.n_items() => self.set_selection(None),
            Some(i) => self.set_selection(Some(i + 1)),
        }
    }

    fn prev_row(&self) {
        match self.selection() {
            None => self.set_selection(Some(self.n_items() - 1)),
            Some(i) if i == 0 => self.set_selection(None),
            Some(i) => self.set_selection(Some(i - 1)),
        }
    }

    fn selected_entry_object(&self) -> Option<EntryObject> {
        self.selection()
            .and_then(|i| self.selection_model().item(i).and_downcast::<EntryObject>())
    }

    fn set_selection(&self, selection: Option<u32>) {
        match selection {
            Some(i) => self.selection_model().select_item(i, true),
            None => self.selection_model().unselect_all(),
        };
    }

    fn selection(&self) -> Option<u32> {
        let bitset = self.selection_model().selection();
        if bitset.is_empty() {
            None
        } else {
            Some(bitset.minimum())
        }
    }

    fn selection_model(&self) -> gtk::SelectionModel {
        self.imp().column_view.model().unwrap()
    }

    fn selection_model_item(&self, i: u32) -> EntryObject {
        self.selection_model()
            .item(i)
            .and_downcast::<EntryObject>()
            .unwrap()
    }

    fn setup_bindings(&self) {
        let imp = self.imp();

        // Show the minor length fields only if applicable
        imp.length_unit_field
            .bind_property("selected", &imp.minor_length_field.get(), "visible")
            .transform_to(|_, i| Some(SizeUnit::from(i).has_minor()))
            .sync_create()
            .build();
        imp.width_unit_field
            .bind_property("selected", &imp.minor_width_field.get(), "visible")
            .transform_to(|_, i| Some(SizeUnit::from(i).has_minor()))
            .sync_create()
            .build();

        // Set the title of the length field based on the unit type
        imp.length_unit_field
            .bind_property("selected", &imp.major_length_field.get(), "title")
            .transform_to(|_, i| Some(SizeUnit::from(i).major_name()))
            .sync_create()
            .build();
        imp.width_unit_field
            .bind_property("selected", &imp.major_width_field.get(), "title")
            .transform_to(|_, i| Some(SizeUnit::from(i).major_name()))
            .sync_create()
            .build();

        imp.dimension_field
            .bind_property("active", &imp.width_group.get(), "visible")
            .transform_to(|_, i: u32| Some(i == 1))
            .sync_create()
            .build();

        self.bind_property("entry-descriptor", &self.column(1), "title")
            .build();
        self.bind_property("entry-descriptor", &imp.properties_group.get(), "title")
            .build();
        self.bind_property("allow-price", &imp.price_field.get(), "visible")
            .sync_create()
            .build();
        self.bind_property("allow-price", &self.column(2), "visible")
            .sync_create()
            .build();
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();
        for field in [
            &imp.material_field,
            &imp.price_field,
            &imp.quantity_field,
            &imp.major_length_field,
            &imp.minor_length_field,
            &imp.major_width_field,
            &imp.minor_width_field,
        ] {
            field.connect_changed(clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.validate_all_entries();
                }
            ));
        }

        // Callback for major-width defined below
        for (field, property_name) in [
            (&imp.material_field, "material"),
            (&imp.name_field, "name"),
            (&imp.price_field, "price"),
            (&imp.quantity_field, "quantity"),
            (&imp.major_length_field, "major-length"),
            (&imp.minor_length_field, "minor-length"),
            (&imp.minor_width_field, "minor-width"),
        ] {
            field.connect_apply(clone!(
                #[weak(rename_to = pane)]
                self,
                move |entry| {
                    if let Some(entry_object) = pane.selected_entry_object() {
                        entry_object.set_property(property_name, entry.text());
                        pane.signal_entries_updated();
                    }
                }
            ));
        }

        for (field, property_name) in [
            (&imp.width_unit_field, "width-unit"),
            (&imp.length_unit_field, "length-unit"),
        ] {
            field.connect_selected_notify(clone!(
                #[weak(rename_to = pane)]
                self,
                move |entry| {
                    if let Some(entry_object) = pane.selected_entry_object() {
                        entry_object.set_property(property_name, entry.selected());
                        pane.signal_entries_updated();
                    }
                }
            ));
        }
        imp.dimension_field.connect_active_notify(clone!(
            #[weak(rename_to = pane)]
            self,
            move |entry| {
                if let Some(entry_object) = pane.selected_entry_object() {
                    if entry.active() == 0 {
                        // Don't update the dimension to 2D until a width has been set
                        entry_object.set_dimension(0);

                        // Clear width fields whenever the dimension is set to 1D
                        entry_object.set_width_unit(pane.default_unit());
                        entry_object.set_major_width("");
                        entry_object.set_minor_width("");

                        pane.signal_entries_updated();

                        // Clear hidden width fields
                        pane.update_fields();
                    }
                }
                pane.validate_all_entries();
            }
        ));
        imp.major_width_field.connect_apply(clone!(
            #[weak(rename_to = pane)]
            self,
            move |entry| {
                if let Some(entry_object) = pane.selected_entry_object() {
                    let imp = pane.imp();
                    if imp.dimension_field.active() == 1 {
                        // Update the dimension to 2D once we have a valid width
                        entry_object.set_dimension(imp.dimension_field.active());

                        entry_object.set_major_width(entry.text());
                        pane.signal_entries_updated();
                    }
                }
            }
        ));
        for field in [&imp.length_unit_field, &imp.width_unit_field] {
            field.connect_selected_notify(clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.validate_all_entries();
                }
            ));
        }
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
                pane.update_can_delete();
            }
        ));
        self.connect_require_quantity_notify(|pane| {
            pane.validate_all_entries();
        });
        self.connect_all_entries_valid_notify(|pane| {
            pane.update_can_add();
        });
        for name in ["default-unit", "default-material-1d", "default-material-2d"] {
            self.connect_notify(Some(name), |pane, _| {
                pane.update_fields();
            });
        }
    }

    fn setup_column_factory(
        &self,
        column_index: u32,
        property: &str,
        sort_property: &str,
        numeric_sort: bool,
    ) {
        // Set up the column factory and sorter
        let factory = gtk::SignalListItemFactory::new();
        let column = self.column(column_index);
        let expression = gtk::PropertyExpression::new(
            EntryObject::static_type(),
            None::<gtk::Expression>,
            sort_property,
        );
        if numeric_sort {
            column.set_sorter(Some(&gtk::NumericSorter::new(Some(&expression))));
        } else {
            column.set_sorter(Some(&gtk::StringSorter::new(Some(&expression))));
        }
        column.set_factory(Some(&factory));

        // Called when widgets need to be created for a new row
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::builder().halign(gtk::Align::Start).build();
            list_item.set_child(Some(&label));
        });

        // Convert to an owned string to allow moving into the closure
        let property = property.to_string();

        // Called when a list item is bound to a row
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry_object = list_item.item().and_downcast::<EntryObject>().unwrap();
            let label = list_item.child().and_downcast::<gtk::Label>().unwrap();

            // Without sync_create() there is no initial update
            entry_object
                .bind_property(&property, &label, "label")
                .sync_create()
                .build();
        });
    }

    fn setup_column_view(&self) {
        // Set up the column view and sorter
        let column_view = &self.imp().column_view;
        let sorter = gtk::SortListModel::builder()
            .model(&self.entries())
            .sorter(&column_view.sorter().unwrap())
            .build();
        let selection = gtk::SingleSelection::builder()
            .model(&sorter)
            .autoselect(false)
            .can_unselect(true)
            .build();
        column_view.set_model(Some(&selection));

        // Define property mappings for each column
        self.setup_column_factory(0, "material", "material", false);
        self.setup_column_factory(1, "name", "name", false);
        self.setup_column_factory(2, "price_display", "price_sort", true);
        self.setup_column_factory(3, "quantity_display", "quantity_sort", true);
        self.setup_column_factory(4, "length_display", "length_sort", true);
        self.setup_column_factory(5, "width_display", "width_sort", true);
    }

    fn setup_entries(&self) {
        let model = Some(ListStore::new::<EntryObject>());
        self.imp().entries.replace(model);
    }

    fn signal_entries_updated(&self) {
        self.emit_by_name::<()>("entries-updated", &[]);
    }

    fn unsort(&self) {
        self.imp()
            .column_view
            .sort_by_column(None, gtk::SortType::Ascending);
    }

    fn update_can_add(&self) {
        self.action_set_enabled("pane.add", self.all_entries_valid());
    }

    fn update_can_delete(&self) {
        self.action_set_enabled("pane.delete", self.selection().is_some());
    }

    fn update_entry_object(&self, entry: &EntryObject) {
        let imp = self.imp();
        entry.set_dimension(imp.dimension_field.active());
        entry.set_name(imp.name_field.text().to_string());
        entry.set_material(imp.material_field.text().to_string());
        entry.set_price(imp.price_field.text().to_string());
        entry.set_quantity(imp.quantity_field.text().to_string());
        entry.set_length_unit(imp.length_unit_field.selected());
        entry.set_major_length(imp.major_length_field.text().to_string());
        if self.use_minor_length() {
            entry.set_minor_length(imp.minor_length_field.text().to_string());
        }
        entry.set_width_unit(imp.width_unit_field.selected());
        if self.use_width() {
            entry.set_major_width(imp.major_width_field.text().to_string());
        }
        if self.use_minor_width() {
            entry.set_minor_width(imp.minor_width_field.text().to_string());
        }
        self.signal_entries_updated();

        // Triggers a re-sort of the column view
        self.imp()
            .column_view
            .sorter()
            .unwrap()
            .changed(gtk::SorterChange::Different);
    }

    fn update_fields(&self) {
        let imp = self.imp();
        match self.selected_entry_object() {
            Some(entry_object) => {
                imp.name_field.set_text(&entry_object.name());
                imp.material_field.set_text(&entry_object.material());
                imp.price_field.set_text(&entry_object.price());
                imp.quantity_field.set_text(&entry_object.quantity());
                imp.major_length_field
                    .set_text(&entry_object.major_length());
                imp.minor_length_field
                    .set_text(&entry_object.minor_length());
                imp.major_width_field.set_text(&entry_object.major_width());
                imp.minor_width_field.set_text(&entry_object.minor_width());

                // Do this after setting other fields to skip the entry animations
                imp.dimension_field.set_active(entry_object.dimension());
                imp.length_unit_field
                    .set_selected(entry_object.length_unit());
                imp.width_unit_field.set_selected(entry_object.width_unit());
            }
            None => {
                for field in [
                    &imp.name_field,
                    &imp.price_field,
                    &imp.quantity_field,
                    &imp.major_length_field,
                    &imp.minor_length_field,
                    &imp.major_width_field,
                    &imp.minor_width_field,
                ] {
                    field.set_text("");
                }
                let material = match imp.dimension_field.active() {
                    0 => self.default_material_1d(),
                    1 => self.default_material_2d(),
                    _ => panic!(),
                };
                imp.material_field.set_text(&material);

                // Do this after setting other fields to skip the entry animations
                imp.length_unit_field.set_selected(self.default_unit());
                imp.width_unit_field.set_selected(self.default_unit());
            }
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

    fn use_minor_length(&self) -> bool {
        SizeUnit::from(self.imp().length_unit_field.selected()).has_minor()
    }

    fn use_minor_width(&self) -> bool {
        self.use_width() && SizeUnit::from(self.imp().width_unit_field.selected()).has_minor()
    }

    fn use_width(&self) -> bool {
        self.imp().dimension_field.active() == 1
    }

    fn validate_all_entries(&self) {
        let mut all_valid = true;
        let imp = self.imp();
        let entry = self.selected_entry_object();
        all_valid &= validate_entry(
            &imp.material_field.get(),
            entry.as_ref().and_then(|e| Some(e.material())),
            |e| e.text_length() != 0,
        );
        all_valid &= validate_entry(
            &imp.name_field.get(),
            entry.as_ref().and_then(|e| Some(e.name())),
            |_| true,
        );
        all_valid &= validate_entry(
            &imp.price_field.get(),
            entry.as_ref().and_then(|e| Some(e.price())),
            |e| parse_price(&e.text(), true).is_ok(),
        );
        all_valid &= validate_entry(
            &imp.quantity_field.get(),
            entry.as_ref().and_then(|e| Some(e.quantity())),
            |e| parse_quantity(&e.text(), !self.require_quantity()).is_ok(),
        );
        all_valid &= validate_entry(
            &imp.major_length_field.get(),
            entry.as_ref().and_then(|e| Some(e.major_length())),
            |e| parse_positive_fraction(&e.text(), false).is_ok(),
        );
        if self.use_minor_length() {
            all_valid &= validate_entry(
                &imp.minor_length_field.get(),
                entry.as_ref().and_then(|e| Some(e.minor_length())),
                |e| parse_positive_fraction(&e.text(), true).is_ok(),
            );
        }
        if self.use_width() {
            all_valid &= validate_entry(
                &imp.major_width_field.get(),
                entry.as_ref().and_then(|e| Some(e.major_width())),
                |e| parse_positive_fraction(&e.text(), false).is_ok(),
            );
        }
        if self.use_minor_width() {
            all_valid &= validate_entry(
                &imp.minor_width_field.get(),
                entry.as_ref().and_then(|e| Some(e.minor_width())),
                |e| parse_positive_fraction(&e.text(), true).is_ok(),
            );
        }
        self.set_all_entries_valid(all_valid);
    }
}
