pub mod components;
pub mod config;
pub mod icons;
pub mod services;
pub mod utils;

use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    adw::{self, prelude::AdwApplicationWindowExt},
    gtk::{
        self,
        glib::object::Cast,
        prelude::{WidgetExt},
    },
    new_action_group, new_stateless_action,
};

use crate::app::{
    components::{
        about_dialog::AboutDialog,
        content::{ContentInput, ContentModel, ContentOutput},
        sidebar::{SidebarModel, SidebarOutput},
    },
};
use crate::app::components::sidebar::SidebarInput;
use crate::app::services::logind::DEFAULT_SEAT;

new_action_group!(pub(super) WindowActionGroup, "win");
new_stateless_action!(AboutAction, WindowActionGroup, "about");
new_stateless_action!(ShortcutsAction, WindowActionGroup, "shortcuts-overlay");
new_stateless_action!(FlushDevicesAction, WindowActionGroup, "flush-devices");

#[derive(Debug)]
pub enum AppInput {
    CollapseSidebar,
    SelectSeat(String),
    RefreshSeats,
}

pub struct App {
    is_expanded: bool,
    sidebar_controller: Controller<SidebarModel>,
    content_controller: Controller<ContentModel>,
    about_dialog_controller: Controller<AboutDialog>,
}

pub struct AppInit {
    pub initialize_styles: Box<dyn FnOnce()>,
}

pub struct AppWidgets {
    split_view: adw::OverlaySplitView,
}

impl SimpleComponent for App {
    type Init = AppInit;
    type Input = AppInput;
    type Output = ();
    type Root = adw::ApplicationWindow;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        adw::ApplicationWindow::builder()
            .width_request(800)
            .height_request(600)
            .default_width(1000)
            .default_height(700)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        (init.initialize_styles)();

        let sidebar_controller = SidebarModel::builder()
            .launch(None)
            .forward(sender.input_sender(), |output| match output {
                SidebarOutput::SelectSeat(target_seat) => AppInput::SelectSeat(target_seat),
            });

        let content_controller =
            ContentModel::builder()
                .launch(DEFAULT_SEAT.to_string())
                .forward(sender.input_sender(), |output| match output {
                    ContentOutput::CollapseSidebar => AppInput::CollapseSidebar,
                    ContentOutput::RefreshSeats => AppInput::RefreshSeats,
                });

        let about_dialog_controller = AboutDialog::builder()
            .launch(root.upcast_ref::<gtk::Window>().clone()) //
            .detach();

        let model = App {
            sidebar_controller,
            content_controller,
            about_dialog_controller,
            is_expanded: true,
        };

        let split_view = adw::OverlaySplitView::builder()
            .enable_show_gesture(true)
            .sidebar_width_fraction(0.3)
            .sidebar(model.sidebar_controller.widget())
            .show_sidebar(model.is_expanded)
            .content(model.content_controller.widget())
            .build();

        root.set_content(Some(&split_view));

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let about_dialog_action = {
            let sender = model.about_dialog_controller.sender().clone();

            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap_or_default();
            })
        };

        let shortcuts_overlay_action = {
            // let shortcuts_overlay = widgets.shortcuts_overlay.clone();
            RelmAction::<ShortcutsAction>::new_stateless(move |_| {
                // shortcuts_overlay.present();
            })
        };

        let flush_devices_action = {
            let sender = model.about_dialog_controller.sender().clone();

            RelmAction::<FlushDevicesAction>::new_stateless(move |_| {
                sender.send(()).unwrap_or_default();
            })
        };

        actions.add_action(about_dialog_action);
        // actions.add_action(shortcuts_overlay_action);
        // actions.add_action(flush_devices_action);

        root.insert_action_group(WindowActionGroup::NAME, Some(&actions.into_action_group()));

        let widgets = AppWidgets { split_view };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::CollapseSidebar => self.is_expanded = !self.is_expanded,
            AppInput::SelectSeat(target_seat) => self
                .content_controller
                .emit(ContentInput::ListSeatDevices(target_seat)),
            AppInput::RefreshSeats => self
                .sidebar_controller
                .emit(SidebarInput::RefreshSeats),
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.split_view.set_show_sidebar(self.is_expanded);
    }
}
