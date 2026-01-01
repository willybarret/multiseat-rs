use crate::app::icons::GtkIcons;
use relm4::adw::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, adw, gtk};

#[derive(Debug)]
pub struct ListDialogComponent {
    pub name: gtk::EntryBuffer,
    pub mode: ListDialogMode,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum ListDialogMode {
    New,
    Edit,
}

#[derive(Debug)]
pub enum ListDialogInput {
    HandleEntry,
}

#[derive(Debug)]
pub enum ListDialogOutput {
    AddSeatToSidebar(String),
    RenameSeat(String),
}

#[relm4::component(pub)]
impl Component for ListDialogComponent {
    type Input = ListDialogInput;
    type Output = ListDialogOutput;
    type Init = Option<String>;
    type CommandOutput = ();

    view! {
        #[root]
        adw::Window {
            set_hide_on_close: true,
            set_default_width: 320,
            set_resizable: false,
            set_modal: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    set_show_end_title_buttons: true,
                    set_css_classes: &["flat"],
                    set_title_widget: Some(&gtk::Box::default())
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 20,
                    set_spacing: 10,
                    gtk::Image {
                            set_icon_size: gtk::IconSize::Large,
                            set_icon_name: Some(match model.mode {
                                ListDialogMode::New => GtkIcons::Add.as_str(),
                                ListDialogMode::Edit => GtkIcons::Edit.as_str(),
                            }),
                    },
                    gtk::Label {
                        set_css_classes: &["title-4"],
                        set_label: match model.mode {
                            ListDialogMode::New => "You're about to add a seat.",
                            ListDialogMode::Edit => "You're about to rename this seat."
                        },
                    },
                    gtk::Label {
                        set_label: "Pick a descriptive name.",
                    },
                    #[name = "new_list_entry"]
                    gtk::Entry {
                        set_placeholder_text: Some("Seat name"),
                        set_buffer: &model.name,
                        connect_activate => ListDialogInput::HandleEntry,
                    },
                    gtk::Button {
                        set_css_classes: &["suggested-action"],
                        set_label: model.label.as_str(),
                        connect_clicked => ListDialogInput::HandleEntry,
                    },
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = if let Some(name) = init {
            ListDialogComponent {
                name: gtk::EntryBuffer::new(Some(name)),
                mode: ListDialogMode::Edit,
                label: "Rename".to_string(),
            }
        } else {
            ListDialogComponent {
                name: gtk::EntryBuffer::new(Some("")),
                mode: ListDialogMode::New,
                label: "Add seat".to_string(),
            }
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            ListDialogInput::HandleEntry => {
                let name = self.name.text();

                match self.mode {
                    ListDialogMode::New => {
                        sender
                            .output(ListDialogOutput::AddSeatToSidebar(name.to_string()))
                            .unwrap_or_default();
                    }
                    ListDialogMode::Edit => {
                        sender
                            .output(ListDialogOutput::RenameSeat(name.to_string()))
                            .unwrap_or_default();
                    }
                }
                root.close();
            }
        }
    }
}
