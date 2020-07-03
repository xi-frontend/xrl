use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AvailableThemes {
    pub themes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeChanged {
    pub name: String,
    pub theme: crate::protocol::ThemeSettings,
}

/// This is a hack to allow PartialEq to be implemented on crate::Message
/// This ignores the theme settings entirely only checking whether the
/// theme names match.
impl PartialEq for ThemeChanged {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
