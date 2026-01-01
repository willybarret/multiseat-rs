pub enum GtkIcons {
    Add,
    Edit,
    Sidebar,
    Reload,
    Menu,
    ViewMore,
    Switch,
    Warning,
    GraphicsCard,
    SoundCard,
    GenericDevice,
    Multitasking,
}

impl GtkIcons {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "list-add-symbolic",
            Self::Edit => "edit-symbolic",
            Self::Sidebar => "sidebar-show-symbolic",
            Self::Reload => "view-refresh-symbolic",
            Self::Menu => "application-menu-symbolic",
            Self::ViewMore => "view-more-symbolic",
            Self::Switch => "applications-other-symbolic",
            Self::Warning => "dialog-warning-symbolic",
            Self::GraphicsCard => "freon-gpu-temperature-symbolic",
            Self::SoundCard => "audio-card-symbolic",
            Self::GenericDevice => "device-notifier-symbolic",
            Self::Multitasking => "org.gnome.Settings-multitasking-symbolic",
        }
    }
}
