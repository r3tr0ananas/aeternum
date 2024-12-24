use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Misc {
    pub custom_models_folder: String
}


impl Default for Misc {
    fn default() -> Self {
        Self {
            custom_models_folder: "".to_string()
        }
    }
}
