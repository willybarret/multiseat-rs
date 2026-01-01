pub enum GTK_ICONS {
    ADD,
    EDIT,
    SIDEBAR,
    RELOAD,
    MENU,
    VIEW_MORE,
    SWITCH,
    WARNING,
    GRAPHICS_CARD,
    SOUND_CARD,
    GENERIC_DEVICE,
    MULTITASKING,
}

impl GTK_ICONS {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ADD => "list-add-symbolic",
            Self::EDIT => "edit-symbolic",
            Self::SIDEBAR => "sidebar-show-symbolic",
            Self::RELOAD => "view-refresh-symbolic",
            Self::MENU => "application-menu-symbolic",
            Self::VIEW_MORE => "view-more-symbolic",
            Self::SWITCH => "applications-other-symbolic",
            Self::WARNING => "dialog-warning-symbolic",
            Self::GRAPHICS_CARD => "freon-gpu-temperature-symbolic",
            Self::SOUND_CARD => "audio-card-symbolic",
            Self::GENERIC_DEVICE => "device-notifier-symbolic",
            Self::MULTITASKING => "org.gnome.Settings-multitasking-symbolic",
        }
    }
}
