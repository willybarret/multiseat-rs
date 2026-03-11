use crate::app::services::logind::DEFAULT_SEAT;
use crate::app::{
    AboutAction, FlushDevicesAction, RefreshAllAction, ShortcutsAction,
    components::list_item::{ListItemInput, ListItemModel, ListItemOutput},
    config::info::APP_NAME,
    icons::GtkIcons,
    utils,
    utils::SeatVariant,
};
use relm4::factory::FactoryVecDeque;
use relm4::{
    ComponentParts, ComponentSender, RelmWidgetExt,
    SimpleComponent, adw,
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt},
    },
};

#[derive(Debug)]
pub struct SidebarModel {
    selected_seat_id: String,
    has_secondary_seats: bool,
    secondary_seats_factory: FactoryVecDeque<ListItemModel>,
    primary_seats_factory: FactoryVecDeque<ListItemModel>,
}

#[derive(Debug)]
pub enum SidebarInput {
    SelectSeat(String, SeatVariant),
    DeleteSeat(String),
    RefreshSeats,
    ResetToDefault,
    OpenIdentifyDialog,
}

#[derive(Debug)]
pub enum SidebarOutput {
    SelectSeat(String),
    OpenIdentifyDialog,
    SeatDeleted(bool, String),
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
                refresh_label => RefreshAllAction,
                flush_devices_label => FlushDevicesAction,
            },
            section! {
                keyboard_shortcuts_label => ShortcutsAction,
                &about_label => AboutAction,
            },
        }
    }

    view! {
        #[root]
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget: Some(&gtk::Label::new(Some("Seats"))),
                pack_start = &gtk::Button {
                    set_icon_name: GtkIcons::Identify.as_str(),
                    set_tooltip: "Identify input device",
                    connect_clicked => SidebarInput::OpenIdentifyDialog,
                },
                pack_end = &gtk::MenuButton {
                    set_tooltip: "Menu",
                    set_icon_name: GtkIcons::Menu.as_str(),
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
                    #[local_ref]
                    primary_list -> gtk::ListBox {
                        set_css_classes: &["boxed-list"],
                        set_margin_horizontal: 10,
                        set_selection_mode: gtk::SelectionMode::None,
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
                        set_selection_mode: gtk::SelectionMode::None,
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
        let refresh_label = "Refresh";

        let mut primary_seats_factory = FactoryVecDeque::<ListItemModel>::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                ListItemOutput::SelectSeat(target_seat, seat_variant) => {
                    SidebarInput::SelectSeat(target_seat, seat_variant)
                }
                ListItemOutput::DeleteSeat(seat_id) => SidebarInput::DeleteSeat(seat_id),
            });

        let mut secondary_seats_factory = FactoryVecDeque::<ListItemModel>::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                ListItemOutput::SelectSeat(target_seat, seat_variant) => {
                    SidebarInput::SelectSeat(target_seat, seat_variant)
                }
                ListItemOutput::DeleteSeat(seat_id) => SidebarInput::DeleteSeat(seat_id),
            });

        for seat in &initial_seats {
            if matches!(seat.variant, SeatVariant::Primary) {
                primary_seats_factory.guard().push_back(seat.clone());
            } else {
                secondary_seats_factory.guard().push_back(seat.clone());
            }
        }

        let model = SidebarModel {
            has_secondary_seats: initial_seats.len() > 1,
            selected_seat_id: DEFAULT_SEAT.to_string(),
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
            SidebarInput::SelectSeat(target_seat, _seat_variant) => {
                self.selected_seat_id = target_seat.clone();
                
                self.primary_seats_factory.broadcast(ListItemInput::UpdateSelection(target_seat.clone()));
                self.secondary_seats_factory.broadcast(ListItemInput::UpdateSelection(target_seat.clone()));
                
                sender
                    .output(SidebarOutput::SelectSeat(target_seat))
                    .unwrap_or_default();
            }
            SidebarInput::DeleteSeat(seat_id) => {
                let success = utils::delete_seat(seat_id.clone());
                
                sender
                    .output(SidebarOutput::SeatDeleted(success, seat_id.clone()))
                    .unwrap_or_default();
                
                if !success {
                    return;
                }

                self.selected_seat_id = DEFAULT_SEAT.to_string();
                
                self.primary_seats_factory.broadcast(ListItemInput::UpdateSelection(DEFAULT_SEAT.to_string()));
                self.secondary_seats_factory.broadcast(ListItemInput::UpdateSelection(DEFAULT_SEAT.to_string()));
                
                sender
                    .output(SidebarOutput::SelectSeat(DEFAULT_SEAT.into()))
                    .unwrap_or_default();

                sender.input(SidebarInput::RefreshSeats);
            }
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
            SidebarInput::ResetToDefault => {
                self.selected_seat_id = DEFAULT_SEAT.to_string();
                
                self.primary_seats_factory.broadcast(ListItemInput::UpdateSelection(DEFAULT_SEAT.to_string()));
                self.secondary_seats_factory.broadcast(ListItemInput::UpdateSelection(DEFAULT_SEAT.to_string()));
                
                sender
                    .output(SidebarOutput::SelectSeat(DEFAULT_SEAT.into()))
                    .unwrap_or_default();

                // Small delay for logind to clean up empty seats after flush
                std::thread::sleep(std::time::Duration::from_millis(100));
                sender.input(SidebarInput::RefreshSeats);
            }
            SidebarInput::OpenIdentifyDialog => {
                sender
                    .output(SidebarOutput::OpenIdentifyDialog)
                    .unwrap_or_default();
            }
        }
    }
}
