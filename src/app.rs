pub mod components;
pub mod config;
pub mod icons;
pub mod lib;

use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    adw::{self},
    gtk::{gio::prelude::ApplicationExt, prelude::GtkWindowExt},
    main_adw_application,
};

use crate::app::components::{
    content::{ContentModel, ContentOutput},
    sidebar::SidebarModel,
};

#[derive(Debug)]
pub enum AppInput {
    CollapseSidebar,
    Quit,
}

pub struct App {
    sidebar_controller: Controller<SidebarModel>,
    content_controller: Controller<ContentModel>,
    collapsed: bool,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            #[name(outter_view)]
            adw::OverlaySplitView {
                #[watch]
                set_collapsed: model.collapsed,
                set_enable_show_gesture: true,
                // set_sidebar_width_fraction: 0.40,

                #[wrap(Some)]
                set_sidebar = model.sidebar_controller.widget(),

                #[wrap(Some)]
                set_content = model.content_controller.widget(),
            },

            set_default_size: (800, 600),
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let sidebar_controller = SidebarModel::builder()
            .launch(None)
            .forward(sender.input_sender(), |output| match output {});

        let content_controller =
            ContentModel::builder()
                .launch("seat0")
                .forward(sender.input_sender(), |output| match output {
                    ContentOutput::CollapseSidebar => AppInput::CollapseSidebar,
                });

        let model = App {
            sidebar_controller,
            content_controller,
            collapsed: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::Quit => main_adw_application().quit(),
            AppInput::CollapseSidebar => self.collapsed = !self.collapsed,
        }
    }
}
