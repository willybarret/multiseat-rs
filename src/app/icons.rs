pub enum GtkIcons {
    ADD,
    SIDEBAR,
    RELOAD,
}

impl GtkIcons {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ADD => "add",
            Self::SIDEBAR => "sidebar",
            Self::RELOAD => "reload",
        }
    }
}