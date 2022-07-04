use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "view1.glade")]
pub struct MainWindow {}

#[glib::object_subclass]
impl ObjectSubclass for MainWindow {
    const NAME: &'static str = "main_window";
    type Type = super::MainWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainWindow {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }
}
impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl BinImpl for MainWindow {}
impl ContainerImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}
