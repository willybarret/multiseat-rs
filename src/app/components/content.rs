use crate::app::components::device_card::{DeviceCard, DeviceCardOutput};
use crate::app::components::input_identify::{InputIdentifyDialog, IdentifyOutput};
use crate::app::icons::GtkIcons;
use crate::app::services::logind::attach_device_to_seat;
use crate::app::utils::filter_devices;
use crate::app::{icons, utils};
use gtk::prelude::ButtonExt;
use gtk::{gdk, glib};
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
    selection_dialog: Controller<SeatSelectionDialog>,
    identify_dialog: Controller<InputIdentifyDialog>,
}

pub type ContentInit = String;

#[derive(Debug)]
pub enum ContentInput {
    CollapseSidebar,
    ListSeatDevices(String),
    AttachDevice(String),
    RefreshContent,
    AttachDeviceToSeat(String, String),
    OpenIdentifyDialog,
}

#[derive(Debug)]
pub enum ContentOutput {
    CollapseSidebar,
    RefreshSeats,
    DeviceSwitched(bool, String), // success, seat_id
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
                pack_start = &gtk::Button {
                    set_icon_name: GtkIcons::Sidebar.as_str(),
                    set_tooltip: "Toggle sidebar",
                    connect_clicked => ContentInput::CollapseSidebar,
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
            SeatSelectionDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    SeatDialogOutput::AttachDeviceToSeat(device, seat) => {
                        ContentInput::AttachDeviceToSeat(device, seat)
                    }
                });

        let identify_dialog =
            InputIdentifyDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    IdentifyOutput::DeviceIdentified(_usb_path) => {
                        ContentInput::RefreshContent
                    }
                    IdentifyOutput::SwitchDevice(usb_path) => {
                        ContentInput::AttachDevice(usb_path)
                    }
                });

        let model = ContentModel {
            selected_seat,
            graphics_cards_factory,
            other_devices_factory,
            selection_dialog,
            identify_dialog,
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

                self.selected_seat = target_seat;
            }
            ContentInput::AttachDevice(device_path) => {
                self.selection_dialog
                    .emit(SeatDialogInput::ShowForDevice(device_path));
            }
            ContentInput::AttachDeviceToSeat(device_path, seat_id) => {
                let result = attach_device_to_seat(&device_path, &seat_id);
                let success = result.is_ok();

                sender.input(ContentInput::ListSeatDevices(self.selected_seat.clone()));
                sender
                    .output(ContentOutput::DeviceSwitched(success, seat_id))
                    .unwrap_or_default();
            }
            ContentInput::RefreshContent => {
                sender.input(ContentInput::ListSeatDevices(self.selected_seat.clone()));
            }
            ContentInput::OpenIdentifyDialog => {
                self.identify_dialog.widget().present();
            }
        }
    }
}

pub struct SeatSelectionDialog {
    target_device: String,
    options: FactoryVecDeque<SeatOption>,
}

#[derive(Debug)]
pub enum SeatDialogInput {
    ShowForDevice(String),
    SeatSelected(String),
    Cancel,
}

#[derive(Debug)]
pub enum SeatDialogOutput {
    AttachDeviceToSeat(String, String),
}

#[relm4::component(pub)]
impl Component for SeatSelectionDialog {
    type Init = ();
    type Input = SeatDialogInput;
    type Output = SeatDialogOutput;
    type CommandOutput = ();

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let options = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |seat_id| {
                SeatDialogInput::SeatSelected(seat_id)
            });

        let model = Self {
            target_device: String::new(),
            options,
        };

        let options_box = model.options.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            SeatDialogInput::ShowForDevice(device) => {
                self.target_device = device;

                let seats = utils::get_seats();

                let mut guard = self.options.guard();
                guard.clear();

                let max_num = seats
                    .iter()
                    .filter_map(|seat| {
                        seat.path
                            .id()
                            .strip_prefix("seat")
                            .and_then(|s| s.parse::<i32>().ok())
                    })
                    .max()
                    .unwrap_or(0);

                let new_id = format!("seat{}", max_num + 1);

                guard.push_back(SeatOptionModel {
                    id: new_id,
                    name: "Create new seat".into(),
                    is_new: true,
                });

                for seat in &seats {
                    guard.push_back(SeatOptionModel {
                        id: seat.path.id().to_string(),
                        name: seat.name.clone(),
                        is_new: false,
                    });
                }

                root.present();
            }
            SeatDialogInput::SeatSelected(seat_id) => {
                root.close();
                sender
                    .output(SeatDialogOutput::AttachDeviceToSeat(
                        self.target_device.clone(),
                        seat_id,
                    ))
                    .unwrap_or_default();
            }
            SeatDialogInput::Cancel => {
                root.close();
            }
        }
    }

    view! {
        #[root]
        adw::Window {
            set_title: Some("Assign to Seat"),
            set_default_width: 380,
            set_default_height: 450,
            set_modal: true,
            set_resizable: false,
            set_hide_on_close: true,

            add_controller = gtk::EventControllerKey::new() {
                connect_key_pressed[sender] => move |_, key, _, _| {
                    if key == gdk::Key::Escape {
                        sender.input(SeatDialogInput::Cancel);
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                },
            },

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_start_title_buttons: false,
                    set_show_end_title_buttons: false,
                    set_css_classes: &["flat"],
                },

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_all: 12,
                        set_spacing: 8,
                        add_css_class: "warning",
                        add_css_class: "card",

                        gtk::Image {
                            set_icon_name: Some("dialog-information-symbolic"),
                        },

                        gtk::Label {
                            set_label: "Assign a GPU first to create a new seat",
                            set_wrap: true,
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                        },
                    },

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 16,
                            set_spacing: 12,

                            gtk::Label {
                                set_label: "Choose a seat to assign this device to:",
                                set_halign: gtk::Align::Start,
                                set_wrap: true,
                                add_css_class: "dim-label",
                            },

                            #[local_ref]
                            options_box -> gtk::ListBox {
                                set_css_classes: &["boxed-list"],
                                set_selection_mode: gtk::SelectionMode::None,
                            },
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_all: 16,
                        set_spacing: 12,
                        set_halign: gtk::Align::End,

                        gtk::Button {
                            set_label: "Cancel",
                            connect_clicked => SeatDialogInput::Cancel,
                        },
                    },
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
struct SeatOptionModel {
    id: String,
    name: String,
    is_new: bool,
}

struct SeatOption {
    model: SeatOptionModel,
}

#[derive(Debug)]
pub enum SeatOptionInput {
    Select,
}

#[relm4::factory]
impl FactoryComponent for SeatOption {
    type Init = SeatOptionModel;
    type Input = SeatOptionInput;
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    fn init_model(init: Self::Init, _: &DynamicIndex, _: FactorySender<Self>) -> Self {
        Self { model: init }
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            SeatOptionInput::Select => {
                sender.output(self.model.id.clone()).unwrap_or_default();
            }
        }
    }

    view! {
        #[root]
        gtk::Button {
            add_css_class: "flat",
            add_css_class: "seat-option",
            connect_clicked => SeatOptionInput::Select,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,
                set_margin_all: 10,

                gtk::Image {
                    set_icon_name: Some(if self.model.is_new {
                        GtkIcons::Add.as_str()
                    } else {
                        icons::GtkIcons::Multitasking.as_str()
                    }),
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: &self.model.name,
                        #[watch]
                        set_css_classes: if self.model.is_new { &["accent"] } else { &[] },
                    },
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: &self.model.id,
                        set_css_classes: &["dim-label", "caption"],
                    },
                },

                gtk::Image {
                    set_icon_name: Some("go-next-symbolic"),
                    add_css_class: "dim-label",
                },
            }
        }
    }
}
