use crate::app::components::delete::{DeleteComponent, DeleteInit, DeleteOutput};
use crate::app::{
    icons::GtkIcons,
    utils::{Seat, SeatVariant},
};
use relm4::factory::{DynamicIndex, FactoryComponent, FactoryView};
use relm4::{
    Component, ComponentController, Controller, FactorySender,
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, GtkWindowExt, ListBoxRowExt, PopoverExt, WidgetExt},
    },
};

#[derive(Debug)]
pub struct ListItemModel {
    seat: Seat,
    is_selected: bool,
}

pub type ListItemInit = Seat;

#[derive(Debug, Clone)]
pub enum ListItemInput {
    UpdateSelection(String),
}

#[derive(Debug)]
pub enum ListItemOutput {
    SelectSeat(String, SeatVariant),
    DeleteSeat(String),
}

pub type ListItemRoot = gtk::ListBoxRow;

pub type ListItemParentWidget = gtk::ListBox;

pub struct ListItemWidgets {
    root: gtk::ListBoxRow,
    #[allow(dead_code)]
    delete_controller: Option<Controller<DeleteComponent>>,
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

    fn init_model(
        seat: <Self as FactoryComponent>::Init,
        _: &<Self as FactoryComponent>::Index,
        _: FactorySender<Self>,
    ) -> Self {
        let is_selected = seat.path.id() == "seat0";
        ListItemModel { seat, is_selected }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let container = gtk::Box::builder()
            .css_classes(vec!["toolbar".to_string()])
            .build();

        let controller = gtk::GestureClick::new();
        let seat_id: String = self.seat.path.id().to_string();
        let seat_variant = self.seat.variant;
        let sender_click = sender.clone();
        controller.connect_pressed(move |_, _, _, _| {
            sender_click
                .output(ListItemOutput::SelectSeat(seat_id.clone(), seat_variant))
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
            .label(&self.seat.name)
            .build();

        let (delete_controller, menu_button) = if matches!(seat_variant, SeatVariant::Secondary) {
            let seat_id_for_delete = self.seat.path.id().to_string();
            let delete = DeleteComponent::builder()
                .launch(DeleteInit {
                    warning: "You are about to delete this seat".into(),
                    delete_warning: "Devices will be attached to the Master seat.".into(),
                })
                .forward(sender.output_sender(), move |message| match message {
                    DeleteOutput::Delete => ListItemOutput::DeleteSeat(seat_id_for_delete.clone()),
                });

            let popover = gtk::Popover::builder()
                .has_arrow(true)
                .build();
            
            let delete_btn = gtk::Button::builder()
                .label("Delete")
                .css_classes(vec!["flat".to_string()])
                .build();
            
            popover.set_child(Some(&delete_btn));
            
            let delete_widget = delete.widget().clone();
            let popover_clone = popover.clone();
            delete_btn.connect_clicked(move |_| {
                popover_clone.popdown();
                delete_widget.present();
            });

            let btn = gtk::MenuButton::builder()
                .visible(true)
                .css_classes(vec!["flat".to_string(), "image-button".to_string()])
                .valign(gtk::Align::Center)
                .icon_name(GtkIcons::ViewMore.as_str())
                .popover(&popover)
                .build();

            (Some(delete), btn)
        } else {
            let btn = gtk::MenuButton::builder()
                .visible(false)
                .build();
            (None, btn)
        };

        plugin_box.append(&label);
        container.append(&plugin_box);
        container.append(&menu_button);
        
        if self.is_selected {
            root.add_css_class("selected-seat");
        }
        root.set_child(Some(&container));

        ListItemWidgets { root, delete_controller }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            ListItemInput::UpdateSelection(selected_id) => {
                self.is_selected = self.seat.path.id() == selected_id;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: FactorySender<Self>) {
        if self.is_selected {
            widgets.root.add_css_class("selected-seat");
        } else {
            widgets.root.remove_css_class("selected-seat");
        }
    }
}
