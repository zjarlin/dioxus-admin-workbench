use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
pub struct CrudPageExtension;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CrudPageConfig {
    pub resource_id: String,
    pub page_size: u32,
}

impl Default for CrudPageConfig {
    fn default() -> Self {
        Self {
            resource_id: String::new(),
            page_size: 20,
        }
    }
}
