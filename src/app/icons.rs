pub enum GtkIcons {
    Add,
    Sidebar,
    Menu,
    ViewMore,
    Switch,
    Warning,
    GraphicsCard,
    SoundCard,
    GenericDevice,
    Multitasking,
    Identify,
    // Vendor icons
    VendorAmd,
    VendorNvidia,
    VendorIntel,
}

impl GtkIcons {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "list-add-symbolic",
            Self::Sidebar => "sidebar-show-symbolic",
            Self::Menu => "application-menu-symbolic",
            Self::ViewMore => "view-more-symbolic",
            Self::Switch => "system-switch-user-symbolic",
            Self::Warning => "dialog-warning-symbolic",
            Self::GraphicsCard => "freon-gpu-temperature-symbolic",
            Self::SoundCard => "audio-card-symbolic",
            Self::GenericDevice => "device-notifier-symbolic",
            Self::Multitasking => "org.gnome.Settings-multitasking-symbolic",
            Self::Identify => "input-keyboard-symbolic",
            Self::VendorAmd => "amd",
            Self::VendorNvidia => "nvidia",
            Self::VendorIntel => "intel",
        }
    }
}

pub fn get_vendor_icon(vendor: &str) -> Option<&'static str> {
    let vendor_lower = vendor.to_lowercase();
    if vendor_lower.contains("amd") || vendor_lower.contains("advanced micro") {
        Some(GtkIcons::VendorAmd.as_str())
    } else if vendor_lower.contains("nvidia") {
        Some(GtkIcons::VendorNvidia.as_str())
    } else if vendor_lower.contains("intel") {
        Some(GtkIcons::VendorIntel.as_str())
    } else {
        None
    }
}
