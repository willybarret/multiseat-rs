use crate::app::components::seat_dialog::{ListDialogComponent, ListDialogOutput};
use crate::app::services::logind::DEFAULT_SEAT;
use crate::app::utils::Seat;
use crate::app::{
    AboutAction, FlushDevicesAction, ShortcutsAction,
    components::list_item::{ListItemModel, ListItemOutput},
    config::info::APP_NAME,
    icons::GTK_ICONS,
    utils,
    utils::SeatVariant,
};
use relm4::adw::prelude::GtkWindowExt;
use relm4::factory::FactoryVecDeque;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw,
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt},
    },
};

#[derive(Debug)]
pub struct SidebarModel {
    selected_seat_variant: SeatVariant,
    has_secondary_seats: bool,
    list_entry: Controller<ListDialogComponent>,
    secondary_seats_factory: FactoryVecDeque<ListItemModel>,
    primary_seats_factory: FactoryVecDeque<ListItemModel>,
}

#[derive(Debug)]
pub enum SidebarInput {
    OpenNewSeatDialog,
    SelectSeat(String, SeatVariant),
    AddSeatToSidebar(String),
    DeleteSeat(String),
    RefreshSeats,
}

#[derive(Debug)]
pub enum SidebarOutput {
    SelectSeat(String),
}

pub type SidebarInit = Option<()>;

#[relm4::component(pub)]
impl SimpleComponent for SidebarModel {
    type Init = SidebarInit;
    type Input = SidebarInput;
    type Output = SidebarOutput;

    menu! {
        primary_menu: {
            section! {
                flush_devices_label => FlushDevicesAction,
                keyboard_shortcuts_label => ShortcutsAction,
                &about_label => AboutAction,
            }
        }
    }

    view! {
        #[root]
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget: Some(&gtk::Label::new(Some("Seats"))),
                pack_start = &gtk::Button {
                    //
                    set_sensitive: false,
                    set_tooltip: "Not implemented yet!",
                    //
                    set_icon_name: GTK_ICONS::ADD.as_str(),
                    connect_clicked => SidebarInput::OpenNewSeatDialog,
                },
                pack_end = &gtk::MenuButton {
                    set_tooltip: "Menu",
                    set_icon_name: GTK_ICONS::MENU.as_str(),
                    set_menu_model: Some(&primary_menu),
                },
            },
            #[wrap(Some)]
            set_content = &gtk::ScrolledWindow {
                set_vexpand: true,
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_top: 5,
                    set_spacing: 10,
                    gtk::Label {
                        set_margin_horizontal: 15,
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["heading"],
                        set_text: "Primary",
                    },
                    // HACK: I can't get set_header working to have different labels per group,
                    // so I'm using two lists instead...
                    #[local_ref]
                    primary_list -> gtk::ListBox {
                        set_css_classes: &["boxed-list"],
                        set_margin_horizontal: 10,
                        #[watch]
                        set_selection_mode: match model.selected_seat_variant {
                            SeatVariant::Primary => gtk::SelectionMode::Single,
                            _ => gtk::SelectionMode::None,
                        },
                    },
                    gtk::Label {
                        set_margin_horizontal: 15,
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["heading"],
                        set_text: "Secondary",
                    },
                    gtk::Label {
                        #[watch]
                        set_visible: !model.has_secondary_seats,
                        set_margin_top: 5,
                        set_margin_horizontal: 15,
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["dim-label"],
                        set_text: "No seats available.",
                    },
                    #[local_ref]
                    secondary_list -> gtk::ListBox {
                        #[watch]
                        set_visible: model.has_secondary_seats,
                        set_css_classes: &["boxed-list"],
                        set_margin_horizontal: 10,
                        #[watch]
                        set_selection_mode: match model.selected_seat_variant {
                            SeatVariant::Secondary => gtk::SelectionMode::Single,
                            _ => gtk::SelectionMode::None,
                        },
                    }
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let initial_seats = utils::get_seats();

        let about_label = format!("About {}", APP_NAME);
        let keyboard_shortcuts_label = "Keyboard Shortcuts";
        let flush_devices_label = "Revert to System Defaults";

        let list_entry =
            ListDialogComponent::builder()
                .launch(None)
                .forward(sender.input_sender(), |message| match message {
                    ListDialogOutput::AddSeatToSidebar(name) => {
                        SidebarInput::AddSeatToSidebar(name)
                    }
                    ListDialogOutput::RenameSeat(_) => todo!(),
                });

        let mut primary_seats_factory = FactoryVecDeque::<ListItemModel>::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                ListItemOutput::SelectSeat(target_seat, seat_variant) => {
                    SidebarInput::SelectSeat(target_seat, seat_variant)
                }
                ListItemOutput::DeleteSeat(seat_id) => SidebarInput::DeleteSeat(seat_id),
            });

        primary_seats_factory.guard().push_back(initial_seats[0].clone());

        let mut secondary_seats_factory = FactoryVecDeque::<ListItemModel>::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                ListItemOutput::SelectSeat(target_seat, seat_variant) => {
                    SidebarInput::SelectSeat(target_seat, seat_variant)
                }
                ListItemOutput::DeleteSeat(seat_id) => SidebarInput::DeleteSeat(seat_id),
            });

        for seat in &initial_seats {
            if matches!(seat.variant, SeatVariant::Secondary) {
                secondary_seats_factory.guard().push_back(seat.clone());
            }
        }

        let model = SidebarModel {
            has_secondary_seats: initial_seats.len() > 1,
            selected_seat_variant: SeatVariant::Primary,
            list_entry,
            primary_seats_factory,
            secondary_seats_factory,
        };

        let primary_list = model.primary_seats_factory.widget();
        let secondary_list = model.secondary_seats_factory.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            // FIXME: When clicking on gtk::MenuButton (aka SelectSeat event), selection mode doesn't work
            SidebarInput::SelectSeat(target_seat, seat_variant) => {
                self.selected_seat_variant = seat_variant;
                sender
                    .output(SidebarOutput::SelectSeat(target_seat))
                    .unwrap_or_default();
            }
            SidebarInput::OpenNewSeatDialog => {
                let list_entry = self.list_entry.widget();
                list_entry.present();
            }
            SidebarInput::AddSeatToSidebar(_) => {}
            SidebarInput::DeleteSeat(seat_id) => {
                if !utils::delete_seat(seat_id) { return }

                self.selected_seat_variant = SeatVariant::Primary;
                sender
                    .output(SidebarOutput::SelectSeat(DEFAULT_SEAT.into()))
                    .unwrap_or_default();

                sender.input(SidebarInput::RefreshSeats);
            },
            SidebarInput::RefreshSeats => {
                let mut guard = self.secondary_seats_factory.guard();
                guard.clear();
                let seats = utils::get_seats();
                self.has_secondary_seats = seats.len() > 1;
                for seat in &seats {
                    if matches!(seat.variant, SeatVariant::Secondary) {
                        guard.push_back(seat.clone());
                    }
                }
            }
        }
    }
}
