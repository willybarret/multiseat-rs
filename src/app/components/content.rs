use crate::app::components::device_card::{DeviceCard, DeviceCardOutput};
use crate::app::services::logind::attach_device_to_seat;
use crate::app::utils::filter_devices;
use crate::app::{icons::GtkIcons, utils};
use gtk::prelude::ButtonExt;
use relm4::adw::prelude::{GtkWindowExt, OrientableExt};
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::prelude::FactoryVecDeque;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, FactorySender,
    RelmWidgetExt, SimpleComponent,
    adw::{self},
    gtk::{
        self,
        prelude::{BoxExt, WidgetExt},
    },
};

pub struct ContentModel {
    selected_seat: String,
    graphics_cards_factory: FactoryVecDeque<DeviceCard>,
    other_devices_factory: FactoryVecDeque<DeviceCard>,
    selection_dialog: Controller<SelectionDialog>,
}

pub type ContentInit = String;

#[derive(Debug)]
pub enum ContentInput {
    CollapseSidebar,
    ListSeatDevices(String),
    AttachDevice(String),
    RefreshContent,
    AttachDeviceToSeat(String, String),
}

#[derive(Debug)]
pub enum ContentOutput {
    CollapseSidebar,
    RefreshSeats,
}

#[relm4::component(pub)]
impl SimpleComponent for ContentModel {
    type Init = ContentInit;
    type Input = ContentInput;
    type Output = ContentOutput;

    view! {
        #[root]
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget: Some(&gtk::Label::new(Some("Devices"))),
                pack_start = &gtk::Box {
                    set_spacing: 6,
                    gtk::Button {
                        set_icon_name: GtkIcons::Sidebar.as_str(),
                        connect_clicked => ContentInput::CollapseSidebar,
                    },
                    gtk::Button {
                        // TODO: Add refresh functionality
                        set_sensitive: false,
                        set_icon_name: GtkIcons::Reload.as_str(),
                        connect_clicked => ContentInput::RefreshContent,
                    }
                },
            },
            #[wrap(Some)]
            set_content = &gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_propagate_natural_height: true,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 20,

                    append = &gtk::Label {
                        set_markup: "<b>GPU Devices</b>",
                        set_halign: gtk::Align::Start,
                        add_css_class: "title-2",
                    },
                    append = &gtk::Label {
                        set_label: "Graphics processors available for rendering and computation.",
                        set_halign: gtk::Align::Start,
                        add_css_class: "dim-label",
                    },
                    #[local_ref]
                    graphics_cards -> gtk::ListBox {
                        add_css_class: "static-list",
                        set_selection_mode: gtk::SelectionMode::None,
                    },

                    append = &gtk::Label {
                        set_markup: "<b>Additional Devices</b>",
                        set_halign: gtk::Align::Start,
                        set_margin_top: 15,
                        add_css_class: "title-2",
                    },
                    append = &gtk::Label {
                        set_label: "Audio interfaces, input devices, and other connected hardware.",
                        set_halign: gtk::Align::Start,
                        add_css_class: "dim-label",
                    },
                    #[local_ref]
                    other_devices -> gtk::ListBox {
                        add_css_class: "static-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        set_show_separators: true,
                    },
                }
            }
        }
    }

    fn init(
        selected_seat: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let devices = utils::get_seat_devices(&selected_seat);
        let (graphics_cards, other_devices) = filter_devices(devices);

        let mut graphics_cards_factory = FactoryVecDeque::<DeviceCard>::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                DeviceCardOutput::AttachDevice(device_path) => {
                    ContentInput::AttachDevice(device_path)
                }
            });

        for gpu in graphics_cards {
            graphics_cards_factory.guard().push_back(gpu.clone());
        }

        let mut other_devices_factory = FactoryVecDeque::<DeviceCard>::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                DeviceCardOutput::AttachDevice(device_path) => {
                    ContentInput::AttachDevice(device_path)
                }
            });

        for device in other_devices {
            other_devices_factory.guard().push_back(device.clone());
        }

        let selection_dialog =
            SelectionDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    DialogOutput::AttachDeviceToSeat(device, seat) => {
                        ContentInput::AttachDeviceToSeat(device, seat)
                    }
                });

        let model = ContentModel {
            selected_seat,
            graphics_cards_factory,
            other_devices_factory,
            selection_dialog,
        };

        let graphics_cards = model.graphics_cards_factory.widget();
        let other_devices = model.other_devices_factory.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ContentInput::CollapseSidebar => sender
                .output(ContentOutput::CollapseSidebar)
                .unwrap_or_default(),
            ContentInput::ListSeatDevices(target_seat) => {
                let devices = utils::get_seat_devices(&target_seat);
                let (graphics_cards, other_devices) = filter_devices(devices);

                self.graphics_cards_factory.guard().clear();
                for gpu in graphics_cards {
                    self.graphics_cards_factory.guard().push_back(gpu.clone());
                }

                self.other_devices_factory.guard().clear();
                for device in other_devices {
                    self.other_devices_factory.guard().push_back(device.clone());
                }
            }
            ContentInput::AttachDevice(device_path) => {
                self.selection_dialog
                    .emit(DialogInput::ShowForDevice(device_path));
                self.selection_dialog.widget().present();
            }
            ContentInput::AttachDeviceToSeat(device_path, seat_id) => {
                println!("{}, {}", device_path, seat_id);

                let _ = attach_device_to_seat(&device_path, &seat_id);

                sender.input(ContentInput::ListSeatDevices(self.selected_seat.clone()));
                sender
                    .output(ContentOutput::RefreshSeats)
                    .unwrap_or_default();
            }
            ContentInput::RefreshContent => todo!(),
        }
    }
}

pub struct SelectionDialog {
    visible: bool,
    target_item: String,
    options: FactoryVecDeque<DialogOption>,
}

#[derive(Debug)]
pub enum DialogInput {
    ShowForDevice(String),
    SeatChosen(String),
}

#[derive(Debug)]
pub enum DialogOutput {
    AttachDeviceToSeat(String, String),
}

#[relm4::component(pub)]
impl Component for SelectionDialog {
    type Init = ();
    type Input = DialogInput;
    type Output = DialogOutput;
    type CommandOutput = ();

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let options = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |device| {
                DialogInput::SeatChosen(device)
            });

        let model = Self {
            visible: false,
            target_item: String::new(),
            options,
        };

        let options_box = model.options.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            DialogInput::ShowForDevice(device) => {
                self.target_item = device;
                self.visible = true;

                let seats = utils::get_seats();

                let mut guard = self.options.guard();
                guard.clear();
                for seat in &seats {
                    guard.push_back(OptionModel {
                        id: seat.path.id().to_string(),
                        name: seat.name.clone(),
                    });
                }

                let max_item = seats
                    .iter()
                    .filter_map(|seat| {
                        seat.name
                            .split_whitespace()
                            .rev()
                            .find_map(|s| s.parse::<i32>().ok())
                            .map(|num| (num, seat))
                    })
                    .max_by_key(|(num, _)| *num);

                let new_id = match max_item {
                    Some((num, _)) => format!("seat{}", num.wrapping_add(1)),
                    None => String::from("seat1"),
                };

                guard.push_front(OptionModel {
                    id: new_id,
                    name: "New seat".into(),
                });
            }
            DialogInput::SeatChosen(option) => {
                self.visible = false;
                sender
                    .output(DialogOutput::AttachDeviceToSeat(
                        self.target_item.clone(),
                        option,
                    ))
                    .unwrap_or_default();
            }
        }
    }

    view! {
        gtk::Window {
            set_title: Some("Select seat"),
            set_modal: true,
            #[watch]
            set_visible: model.visible,
            set_default_size: (300, 400),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 10,

                #[local_ref]
                options_box -> gtk::ListBox {
                    set_vexpand: true,
                }
            }
        }
    }
}
#[derive(Debug)]
pub enum DialogOptionInput {
    Clicked,
}

struct OptionModel {
    id: String,
    name: String,
}
struct DialogOption(OptionModel);

#[relm4::factory]
impl FactoryComponent for DialogOption {
    type Init = OptionModel;
    type Input = DialogOptionInput;
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    fn init_model(init: Self::Init, _: &DynamicIndex, _: FactorySender<Self>) -> Self {
        Self(init)
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            DialogOptionInput::Clicked => {
                sender.output(self.0.id.clone()).unwrap_or_default();
            }
        }
    }

    view! {
        #[root]
        gtk::Button {
            add_css_class: "flat",
            set_label: &self.0.name,
            connect_clicked => DialogOptionInput::Clicked,
        }
    }
}
