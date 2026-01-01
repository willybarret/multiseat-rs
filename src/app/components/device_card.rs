use crate::app::icons::GtkIcons;
use crate::app::utils::{Device, Port};
use relm4::adw::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::factory::{DynamicIndex, FactoryComponent, FactoryVecDeque, FactoryView};
use relm4::gtk::pango::EllipsizeMode;
use relm4::{FactorySender, RelmWidgetExt, gtk};

pub struct DeviceCard {
    device: Device,
    ports_factory: FactoryVecDeque<PortComponent>,
}

#[derive(Debug)]
pub enum DeviceCardInput {
    AttachDevice,
}

#[derive(Debug)]
pub enum DeviceCardOutput {
    AttachDevice(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for DeviceCard {
    type Init = Device;
    type Input = DeviceCardInput;
    type Output = DeviceCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;
    type Widgets = DeviceCardWidgets;

    fn init_model(device: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let mut ports_factory = FactoryVecDeque::<PortComponent>::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        for port in device.clone().ports.unwrap_or_default() {
            ports_factory.guard().push_back(port);
        }

        Self {
            device,
            ports_factory,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let ports_box = self.ports_factory.widget();
        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            DeviceCardInput::AttachDevice => sender
                .output(DeviceCardOutput::AttachDevice(self.device.path.clone()))
                .unwrap_or_default(),
        }
    }

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 12,
            set_focusable: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,

                gtk::Image {
                    set_icon_name: Some(self.device.variant.icon_name()),
                    set_pixel_size: 48,
                    set_valign: gtk::Align::Center,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    set_hexpand: true,


                    gtk::Label {
                        set_ellipsize: EllipsizeMode::End,
                        set_use_markup: true,
                        #[watch]
                        set_label: &format!("<b>Tag:</b> {}", self.device.sys_name),
                        set_tooltip: &self.device.vendor,
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        set_ellipsize: EllipsizeMode::End,
                        set_use_markup: true,
                        #[watch]
                        set_label: &format!("<b>Vendor:</b> {}", self.device.vendor),
                        set_tooltip: &self.device.vendor,
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        set_ellipsize: EllipsizeMode::End,
                        set_use_markup: true,
                        #[watch]
                        set_label: &format!("<b>Model:</b> {}", self.device.model),
                        set_tooltip: &self.device.model,
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        set_ellipsize: EllipsizeMode::End,
                        set_use_markup: true,
                        #[watch]
                        set_label: &format!("<b>Path:</b> {}", self.device.path),
                        set_tooltip: &self.device.path,
                        set_halign: gtk::Align::Start,
                        add_css_class: "dim-label",
                        add_css_class: "caption",
                    },
                },

                gtk::Button {
                    set_icon_name: GtkIcons::Switch.as_str(),
                    set_valign: gtk::Align::Center,
                    connect_clicked => DeviceCardInput::AttachDevice,
                },
            },

            gtk::Box {
                // TODO: Implementation for DRM leases required first.
                set_visible: self.device.ports.is_some(),
                set_orientation: gtk::Orientation::Vertical,

                gtk::Separator {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_top: 4,
                    set_margin_bottom: 4,
                },

                #[local_ref]
                ports_box -> gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_selection_mode: gtk::SelectionMode::None,
                    set_column_spacing: 8,
                    set_row_spacing: 8,
                    set_halign: gtk::Align::Start,

                }
            }
        }
    }
}

struct PortComponent {
    port: Port,
}

#[relm4::factory(pub)]
impl FactoryComponent for PortComponent {
    type Init = Port;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { port: init }
    }

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_label: &self.port.name,
            set_sensitive: false,
            set_tooltip_text: Some("Not implemented yet"),
        }
    }
}
