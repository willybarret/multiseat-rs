use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, adw,
    gtk::{self, prelude::GtkWindowExt},
};

use crate::app::config::info::{APP_DESCRIPTION, APP_NAME, DEV_NAME, ISSUE_URL, VERSION, WEBSITE};
use crate::app::icons::GtkIcons;

type MainWindow = gtk::Window;

pub struct AboutDialog {}

pub struct AboutDialogWidgets {
    main_window: MainWindow,
}

impl SimpleComponent for AboutDialog {
    type Input = ();
    type Output = ();
    type Init = MainWindow;
    type Root = ();
    type Widgets = AboutDialogWidgets;

    fn init_root() -> Self::Root {}

    fn init(
        main_window: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = AboutDialogWidgets { main_window };

        ComponentParts { widgets, model }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let dialog = adw::AboutWindow::builder()
            .application_name(APP_NAME)
            // .icon_name(APP_ID)
            // .application_icon(APP_ID)
            .icon_name(GtkIcons::Multitasking.as_str())
            .application_icon(GtkIcons::Multitasking.as_str())
            .version(VERSION)
            .developer_name(DEV_NAME)
            .website(WEBSITE)
            .comments(APP_DESCRIPTION)
            .license_type(gtk::License::Gpl30)
            .issue_url(ISSUE_URL)
            .modal(true)
            .transient_for(&widgets.main_window)
            .build();

        dialog.present();
    }
}
