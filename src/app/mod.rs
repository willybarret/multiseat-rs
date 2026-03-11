pub mod components;
pub mod config;
pub mod icons;
pub mod services;
pub mod utils;

use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    adw::{self, prelude::AdwApplicationWindowExt},
    gtk::{self, glib::object::Cast, prelude::WidgetExt},
    new_action_group, new_stateless_action,
};

use crate::app::components::sidebar::SidebarInput;
use crate::app::components::{
    about_dialog::AboutDialog,
    content::{ContentInput, ContentModel, ContentOutput},
    sidebar::{SidebarModel, SidebarOutput},
};
use crate::app::services::logind::{self, DEFAULT_SEAT};

new_action_group!(pub(super) WindowActionGroup, "win");
new_stateless_action!(AboutAction, WindowActionGroup, "about");
new_stateless_action!(ShortcutsAction, WindowActionGroup, "shortcuts-overlay");
new_stateless_action!(FlushDevicesAction, WindowActionGroup, "flush-devices");
new_stateless_action!(RefreshAllAction, WindowActionGroup, "refresh-all");

#[derive(Debug)]
pub enum AppInput {
    CollapseSidebar,
    SelectSeat(String),
    RefreshSeats,
    RefreshAll,
    ShowToast(String),
    FlushDevicesResult(bool),
    DeviceSwitchResult(bool, String),
    SeatDeleteResult(bool, String),
    OpenIdentifyDialog,
}

pub struct App {
    is_expanded: bool,
    sidebar_controller: Controller<SidebarModel>,
    content_controller: Controller<ContentModel>,
    about_dialog_controller: Controller<AboutDialog>,
    toast_overlay: adw::ToastOverlay,
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

        let sidebar_controller =
            SidebarModel::builder()
                .launch(None)
                .forward(sender.input_sender(), |output| match output {
                    SidebarOutput::SelectSeat(target_seat) => AppInput::SelectSeat(target_seat),
                    SidebarOutput::OpenIdentifyDialog => AppInput::OpenIdentifyDialog,
                    SidebarOutput::SeatDeleted(success, seat) => AppInput::SeatDeleteResult(success, seat),
                });

        let content_controller = ContentModel::builder()
            .launch(DEFAULT_SEAT.to_string())
            .forward(sender.input_sender(), |output| match output {
                ContentOutput::CollapseSidebar => AppInput::CollapseSidebar,
                ContentOutput::RefreshSeats => AppInput::RefreshSeats,
                ContentOutput::DeviceSwitched(success, seat) => AppInput::DeviceSwitchResult(success, seat),
            });

        let about_dialog_controller = AboutDialog::builder()
            .launch(root.upcast_ref::<gtk::Window>().clone()) //
            .detach();

        let toast_overlay = adw::ToastOverlay::new();

        let split_view = adw::OverlaySplitView::builder()
            .enable_show_gesture(true)
            .sidebar_width_fraction(0.3)
            .show_sidebar(true)
            .build();

        let model = App {
            sidebar_controller,
            content_controller,
            about_dialog_controller,
            is_expanded: true,
            toast_overlay: toast_overlay.clone(),
        };

        split_view.set_sidebar(Some(model.sidebar_controller.widget()));
        split_view.set_content(Some(model.content_controller.widget()));
        
        toast_overlay.set_child(Some(&split_view));
        root.set_content(Some(&toast_overlay));

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let about_dialog_action = {
            let sender = model.about_dialog_controller.sender().clone();

            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap_or_default();
            })
        };

        let _shortcuts_overlay_action = {
            RelmAction::<ShortcutsAction>::new_stateless(move |_| {
                // TODO: shortcuts_overlay.present();
            })
        };

        let flush_devices_action = {
            let app_sender = sender.input_sender().clone();

            RelmAction::<FlushDevicesAction>::new_stateless(move |_| {
                let success = logind::flush_devices().is_ok();
                app_sender.send(AppInput::FlushDevicesResult(success)).unwrap_or_default();
            })
        };

        let refresh_all_action = {
            let app_sender = sender.input_sender().clone();

            RelmAction::<RefreshAllAction>::new_stateless(move |_| {
                app_sender.send(AppInput::RefreshAll).unwrap_or_default();
            })
        };

        actions.add_action(about_dialog_action);
        actions.add_action(flush_devices_action);
        actions.add_action(refresh_all_action);

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
            AppInput::RefreshSeats => self.sidebar_controller.emit(SidebarInput::RefreshSeats),
            AppInput::RefreshAll => {
                self.sidebar_controller.emit(SidebarInput::RefreshSeats);
                self.content_controller.emit(ContentInput::RefreshContent);
            }
            AppInput::ShowToast(message) => {
                let toast = adw::Toast::new(&message);
                self.toast_overlay.add_toast(toast);
            }
            AppInput::FlushDevicesResult(success) => {
                let message = if success {
                    "Device assignments reset to system defaults"
                } else {
                    "Failed to reset device assignments"
                };
                let toast = adw::Toast::new(message);
                self.toast_overlay.add_toast(toast);
                self.sidebar_controller.emit(SidebarInput::ResetToDefault);
            }
            AppInput::DeviceSwitchResult(success, seat) => {
                let message = if success {
                    format!("Device moved to {}", seat)
                } else {
                    format!("Failed to move device to {}", seat)
                };
                let toast = adw::Toast::new(&message);
                self.toast_overlay.add_toast(toast);
                if success {
                    self.sidebar_controller.emit(SidebarInput::RefreshSeats);
                    self.content_controller.emit(ContentInput::RefreshContent);
                }
            }
            AppInput::SeatDeleteResult(success, seat_id) => {
                let message = if success {
                    format!("Seat {} deleted", seat_id)
                } else {
                    format!("Failed to delete seat {}", seat_id)
                };
                let toast = adw::Toast::new(&message);
                self.toast_overlay.add_toast(toast);
            }
            AppInput::OpenIdentifyDialog => {
                self.content_controller.emit(ContentInput::OpenIdentifyDialog);
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.split_view.set_show_sidebar(self.is_expanded);
    }
}
