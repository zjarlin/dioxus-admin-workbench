use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceCatalog {
    #[serde(default)]
    pub resources: BTreeMap<String, ResourceDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub id: String,
    pub name: String,
    pub title: String,
    #[serde(default = "default_id_field")]
    pub id_field: String,
    #[serde(default)]
    pub fields: Vec<ResourceFieldDefinition>,
    #[serde(default)]
    pub operations: ResourceOperations,
}

fn default_id_field() -> String {
    "id".to_owned()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceFieldDefinition {
    pub name: String,
    pub title: String,
    pub kind: ResourceFieldKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "visible_by_default")]
    pub list_visible: bool,
    #[serde(default = "visible_by_default")]
    pub form_visible: bool,
}

const fn visible_by_default() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFieldKind {
    Text,
    Integer,
    Decimal,
    Boolean,
    Timestamp,
    Json,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceOperations {
    pub list: bool,
    pub create: bool,
    pub update: bool,
    pub delete: bool,
}

pub type ResourceRecord = BTreeMap<String, Value>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourcePage {
    pub items: Vec<ResourceRecord>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ResourceRequest {
    List {
        resource_id: String,
        page: u32,
        page_size: u32,
    },
    Create {
        resource_id: String,
        values: ResourceRecord,
    },
    Update {
        resource_id: String,
        record_id: String,
        values: ResourceRecord,
    },
    Delete {
        resource_id: String,
        record_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ResourceResponse {
    Page(ResourcePage),
    Record(ResourceRecord),
    Deleted,
}

impl ResourceCatalog {
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ResourceDefinition> {
        self.resources.get(id)
    }
}
