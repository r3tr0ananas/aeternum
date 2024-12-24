use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyBinds {
    pub about_box: String,
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            about_box: "A".to_string()
        }
    }
}
