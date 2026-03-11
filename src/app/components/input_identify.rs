use crate::app::icons::GtkIcons;
use crate::app::services::input::{self, IdentifiedDevice};
use gtk::gdk;
use gtk::glib;
use relm4::adw::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, adw, gtk};
use std::thread;

#[derive(Debug, Clone)]
pub enum IdentifyState {
    Ready,
    Listening,
    Found(IdentifiedDevice),
    Error(String),
}

pub struct InputIdentifyDialog {
    state: IdentifyState,
}

#[derive(Debug)]
pub enum IdentifyInput {
    StartListening,
    DeviceIdentified(IdentifiedDevice),
    IdentificationFailed(String),
    Close,
    Reset,
    SwitchToSeat,
}

#[derive(Debug)]
pub enum IdentifyOutput {
    DeviceIdentified(String),
    SwitchDevice(String),
}

#[relm4::component(pub)]
impl Component for InputIdentifyDialog {
    type Init = ();
    type Input = IdentifyInput;
    type Output = IdentifyOutput;
    type CommandOutput = IdentifyWorkerOutput;

    fn init(_init: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self {
            state: IdentifyState::Ready,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            IdentifyInput::StartListening => {
                self.state = IdentifyState::Listening;

                sender.command(|out, shutdown| {
                    shutdown
                        .register(async move {
                            // Running the process in a separate thread:
                            let result = thread::spawn(|| {
                                input::identify_input_device(10_000) // 10 second timeout
                            })
                            .join();

                            match result {
                                Ok(Some(device)) => {
                                    let _ = out.send(IdentifyWorkerOutput::Found(device));
                                }
                                Ok(None) => {
                                    let _ = out.send(IdentifyWorkerOutput::Timeout);
                                }
                                Err(_) => {
                                    let _ = out.send(IdentifyWorkerOutput::Error(
                                        "Thread panicked".into(),
                                    ));
                                }
                            }
                        })
                        .drop_on_shutdown()
                });
            }
            IdentifyInput::DeviceIdentified(device) => {
                let usb_path = device.usb_device_path.clone();
                self.state = IdentifyState::Found(device);
                sender
                    .output(IdentifyOutput::DeviceIdentified(usb_path))
                    .unwrap_or_default();
            }
            IdentifyInput::IdentificationFailed(error) => {
                self.state = IdentifyState::Error(error);
            }
            IdentifyInput::Close => {
                self.state = IdentifyState::Ready;
                root.close();
            }
            IdentifyInput::Reset => {
                self.state = IdentifyState::Ready;
            }
            IdentifyInput::SwitchToSeat => {
                if let IdentifyState::Found(ref device) = self.state {
                    let usb_path = device.usb_device_path.clone();
                    root.close();
                    self.state = IdentifyState::Ready;
                    sender
                        .output(IdentifyOutput::SwitchDevice(usb_path))
                        .unwrap_or_default();
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            IdentifyWorkerOutput::Found(device) => {
                sender.input(IdentifyInput::DeviceIdentified(device));
            }
            IdentifyWorkerOutput::Timeout => {
                sender.input(IdentifyInput::IdentificationFailed(
                    "Timeout: No input detected within 10 seconds".into(),
                ));
            }
            IdentifyWorkerOutput::Error(msg) => {
                sender.input(IdentifyInput::IdentificationFailed(msg));
            }
        }
    }

    view! {
        #[root]
        adw::Window {
            set_title: Some("Identify Input Device"),
            set_default_width: 420,
            set_default_height: 380,
            set_modal: true,
            set_resizable: false,
            set_hide_on_close: true,

            add_controller = gtk::EventControllerKey::new() {
                connect_key_pressed[sender] => move |_, key, _, _| {
                    if key == gdk::Key::Escape {
                        sender.input(IdentifyInput::Close);
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                },
            },

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: true,
                    set_css_classes: &["flat"],
                    pack_start = &gtk::Button {
                        set_label: "Close",
                        #[watch]
                        set_sensitive: !matches!(model.state, IdentifyState::Listening),
                        connect_clicked => IdentifyInput::Close,
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 24,
                    set_spacing: 16,
                    set_valign: gtk::Align::Center,

                    // Ready state
                    gtk::Box {
                        #[watch]
                        set_visible: matches!(model.state, IdentifyState::Ready),
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        set_halign: gtk::Align::Center,

                        gtk::Image {
                            set_icon_name: Some(GtkIcons::Identify.as_str()),
                            set_pixel_size: 64,
                            add_css_class: "dim-label",
                        },
                        gtk::Label {
                            set_label: "Press any key or use any input device to identify it.",
                            set_wrap: true,
                            set_justify: gtk::Justification::Center,
                            add_css_class: "dim-label",
                        },
                        gtk::Button {
                            set_label: "Start Identification",
                            add_css_class: "suggested-action",
                            connect_clicked => IdentifyInput::StartListening,
                        },
                    },

                    gtk::Box {
                        #[watch]
                        set_visible: matches!(model.state, IdentifyState::Listening),
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        set_halign: gtk::Align::Center,

                        gtk::Spinner {
                            set_spinning: true,
                            set_width_request: 48,
                            set_height_request: 48,
                        },
                        gtk::Label {
                            set_label: "Listening for input...",
                            add_css_class: "title-2",
                        },
                        gtk::Label {
                            set_label: "Press any key or gamepad button.",
                            set_wrap: true,
                            set_justify: gtk::Justification::Center,
                            add_css_class: "dim-label",
                        },
                    },

                    gtk::Box {
                        #[watch]
                        set_visible: matches!(model.state, IdentifyState::Found(_)),
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,
                        set_halign: gtk::Align::Fill,

                        gtk::Image {
                            set_icon_name: Some("emblem-ok-symbolic"),
                            set_pixel_size: 48,
                            add_css_class: "success",
                        },
                        gtk::Label {
                            set_label: "Device Identified!",
                            add_css_class: "title-2",
                        },
                        gtk::Box {
                            set_css_classes: &["card"],
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_top: 8,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_margin_all: 12,
                                set_spacing: 12,

                                gtk::Image {
                                    set_icon_name: Some(GtkIcons::GenericDevice.as_str()),
                                    set_pixel_size: 32,
                                },
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 4,
                                    set_hexpand: true,

                                    gtk::Label {
                                        #[watch]
                                        set_label: &match &model.state {
                                            IdentifyState::Found(d) => d.name.clone(),
                                            _ => String::new(),
                                        },
                                        set_halign: gtk::Align::Start,
                                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: &match &model.state {
                                            IdentifyState::Found(d) => format!("USB Device: {}", d.usb_device_name),
                                            _ => String::new(),
                                        },
                                        set_halign: gtk::Align::Start,
                                        add_css_class: "dim-label",
                                        add_css_class: "caption",
                                    },
                                },
                            },
                        },
                        gtk::Label {
                            #[watch]
                            set_markup: &match &model.state {
                                IdentifyState::Found(d) => format!(
                                    "Look for <b>{}</b> in the device list to switch it to a different seat.",
                                    d.usb_device_name
                                ),
                                _ => String::new(),
                            },
                            set_wrap: true,
                            set_justify: gtk::Justification::Center,
                            add_css_class: "dim-label",
                            set_margin_top: 8,
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            set_halign: gtk::Align::Center,
                            set_margin_top: 8,

                            gtk::Button {
                                set_label: "Identify Another",
                                connect_clicked => IdentifyInput::Reset,
                            },
                            gtk::Button {
                                set_label: "Switch to Seat",
                                add_css_class: "suggested-action",
                                connect_clicked => IdentifyInput::SwitchToSeat,
                            },
                        },
                    },

                    gtk::Box {
                        #[watch]
                        set_visible: matches!(model.state, IdentifyState::Error(_)),
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,
                        set_halign: gtk::Align::Center,

                        gtk::Image {
                            set_icon_name: Some(GtkIcons::Warning.as_str()),
                            set_pixel_size: 48,
                            add_css_class: "warning",
                        },
                        gtk::Label {
                            #[watch]
                            set_label: &match &model.state {
                                IdentifyState::Error(e) => e.clone(),
                                _ => String::new(),
                            },
                            set_wrap: true,
                            set_justify: gtk::Justification::Center,
                            add_css_class: "dim-label",
                        },
                        gtk::Button {
                            set_label: "Try Again",
                            add_css_class: "suggested-action",
                            connect_clicked => IdentifyInput::Reset,
                        },
                    },
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum IdentifyWorkerOutput {
    Found(IdentifiedDevice),
    Timeout,
    Error(String),
}
