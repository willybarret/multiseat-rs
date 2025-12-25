use crate::app::{
    icons::GtkIcons,
    lib::udev_manager::{get_seat_devices},
};
use gtk::prelude::ButtonExt;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
    adw::{self},
    gtk::{self},
};
use udev::Device;

pub struct ContentModel {
    devices: Vec<Device>,
}

#[derive(Debug)]
pub enum ContentInput {
    CollapseSidebar,
}

#[derive(Debug)]
pub enum ContentOutput {
    CollapseSidebar,
}

#[relm4::component(pub)]
impl SimpleComponent for ContentModel {
    type Init = &'static str;
    type Input = ContentInput;
    type Output = ContentOutput;

    view! {
        #[root]
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget: Some(&gtk::Label::new(Some("Devices"))),
                pack_start = &gtk::Button {
                    set_icon_name: GtkIcons::SIDEBAR.as_str(),
                    connect_clicked => ContentInput::CollapseSidebar,
                },
                pack_end = &gtk::Button {
                    set_icon_name: GtkIcons::RELOAD.as_str(),
                    // connect_clicked => ContentOutput::RefreshContent,
                }
            },
            #[wrap(Some)]
            set_content = &gtk::ScrolledWindow {
                #[wrap(Some)]
                set_child = &gtk::Label {
                    set_text: &get_devices_as_str(&model.devices),
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let seat = init;
        let devices = get_seat_devices(seat);

        let model = ContentModel { devices };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ContentInput::CollapseSidebar => sender
                .output(ContentOutput::CollapseSidebar)
                .unwrap_or_default(),
        }
    }
}

fn get_devices_as_str(devices: &Vec<Device>) -> String {
    devices
        .iter()
        .filter_map(|d| d.syspath().to_str())
        .collect::<Vec<_>>()
        .join("\n")
}
