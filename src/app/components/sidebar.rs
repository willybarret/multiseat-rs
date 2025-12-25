use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, adw,
    gtk::{self, prelude::ButtonExt},
};

use crate::app::icons::GtkIcons;

pub struct SidebarModel {
    seats: Option<()>,
}

#[derive(Debug)]
pub enum SidebarInput {}

#[derive(Debug)]
pub enum SidebarOutput {}

#[relm4::component(pub)]
impl SimpleComponent for SidebarModel {
    type Init = Option<()>;
    type Input = SidebarInput;
    type Output = SidebarOutput;

    view! {
        #[root]
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget: Some(&gtk::Label::new(Some("Seats"))),
                pack_start = &gtk::Button {
                    set_icon_name: GtkIcons::ADD.as_str(),
                }
            },
            #[wrap(Some)]
            set_content = &gtk::Box {

            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SidebarModel { seats: None };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
