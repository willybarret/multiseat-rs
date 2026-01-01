use crate::app::{
    icons::GTK_ICONS,
    utils::{Seat, SeatVariant},
};
use relm4::actions::ActionName;
use relm4::adw::gio;
use relm4::{actions::{ActionGroupName, RelmAction, RelmActionGroup}, gtk::{
    self,
    prelude::{BoxExt, ListBoxRowExt, WidgetExt},
}, new_action_group, new_stateless_action, Component, ComponentController, FactorySender};
use relm4::adw::prelude::GtkWindowExt;
use relm4::factory::{DynamicIndex, FactoryComponent, FactoryView};
use crate::app::components::delete::{DeleteComponent, DeleteInit, DeleteOutput};

new_action_group!(pub(super) ListItemActionGroup, "list-item");
new_stateless_action!(DeleteSeatAction, ListItemActionGroup, "delete-seat");
new_stateless_action!(RenameSeatAction, ListItemActionGroup, "rename-seat");

#[derive(Debug)]
pub struct ListItemModel {
    seat: Seat,
}

pub type ListItemInit = Seat;

#[derive(Debug)]
pub enum ListItemInput {
    // SelectSeat,
}

#[derive(Debug)]
pub enum ListItemOutput {
    SelectSeat(String, SeatVariant),
    DeleteSeat(String),
}

pub type ListItemRoot = gtk::ListBoxRow;

pub type ListItemParentWidget = gtk::ListBox;

#[derive(Debug)]
pub struct ListItemWidgets {
    container: gtk::Box,
}

impl FactoryComponent for ListItemModel {
    type Index = DynamicIndex;
    type ParentWidget = ListItemParentWidget;
    type Init = ListItemInit;
    type Input = ListItemInput;
    type Output = ListItemOutput;
    type Widgets = ListItemWidgets;
    type Root = ListItemRoot;
    type CommandOutput = ();

    fn init_root(&self) -> <Self as FactoryComponent>::Root {
        gtk::ListBoxRow::builder().build()
    }


    fn init_model(seat: <Self as FactoryComponent>::Init, _: &<Self as FactoryComponent>::Index, _: FactorySender<Self>) -> Self {
        ListItemModel { seat }
    }

    fn init_widgets(&mut self, index: &Self::Index, root: Self::Root, returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget, sender: FactorySender<Self>) -> Self::Widgets {
        let seat_id = self.seat.path.id().to_string();
        let delete = DeleteComponent::builder()
            .launch(DeleteInit {
                warning: "You are about to delete this seat".into(),
                delete_warning: "Devices will be attached to the Master seat.".into(),
            })
            .forward(sender.output_sender(), move |message| match message {
                DeleteOutput::Delete => ListItemOutput::DeleteSeat(seat_id.clone()),
            });

        // let seat_id: String = self.seat.path.id().to_string();
        // let sender_activate = sender.clone();
        // root.connect_activate(move |root| {
        //     sender_activate
        //         .output(ListItemOutput::SelectSeat(
        //             seat_id.clone(),
        //             self.seat.variant,
        //         ))
        //         .unwrap_or_default();
        // });

        let container = gtk::Box::builder()
            .css_classes(vec!["toolbar".to_string()])
            .build();

        let controller = gtk::GestureClick::new();
        let seat_id: String = self.seat.path.id().to_string();
        let seat_variant = self.seat.variant;
        // let sender_pressed = sender.clone();
        controller.connect_pressed(move |_, _, _, _| {
            // sender_pressed
            sender
                .output(ListItemOutput::SelectSeat(
                    seat_id.clone(),
                    seat_variant,
                ))
                .unwrap_or_default();
        });

        container.add_controller(controller);

        let plugin_box = gtk::Box::builder()
            .css_classes(vec!["plugin".to_string()])
            .build();

        let label = gtk::Label::builder()
            .hexpand(true)
            .halign(gtk::Align::Start)
            .wrap(true)
            .natural_wrap_mode(gtk::NaturalWrapMode::Word)
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(5)
            .margin_end(5)
            // watch label? (which can be renamed)
            .label(&self.seat.name)
            .build();

        let menu_model = gio::Menu::new();
        menu_model.append(Some("Rename"), Some(&RenameSeatAction::action_name()));
        menu_model.append(Some("Delete"), Some(&DeleteSeatAction::action_name()));

        let list_actions = gtk::MenuButton::builder()
            .visible(matches!(seat_variant, SeatVariant::Secondary))
            .css_classes(vec!["flat".to_string(), "image-button".to_string()])
            .valign(gtk::Align::Center)
            .icon_name(GTK_ICONS::VIEW_MORE.as_str())
            .menu_model(&menu_model)
            .build();

        plugin_box.append(&label);
        container.append(&plugin_box);
        container.append(&list_actions);
        root.set_child(Some(&container));

        let mut actions = RelmActionGroup::<ListItemActionGroup>::new();

        let delete_seat_action = {
            RelmAction::<DeleteSeatAction>::new_stateless(move |_| {
                delete.widget().present()
            })
        };

        actions.add_action(delete_seat_action);

        root.insert_action_group(
            ListItemActionGroup::NAME,
            Some(&actions.into_action_group()),
        );

        ListItemWidgets { container }
    }
}
