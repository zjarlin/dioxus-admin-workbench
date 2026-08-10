use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefinitionId(String);

impl DefinitionId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    #[must_use]
    pub fn from_value(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DefinitionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DefinitionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub scenes: Vec<SceneDefinition>,
    #[serde(default)]
    pub pages: Vec<PageDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub menus: Vec<MenuDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_id: Option<DefinitionId>,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    #[serde(default)]
    pub children: Vec<MenuDefinition>,
}

const fn enabled_by_default() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub title: String,
    pub renderer: PageRendererDefinition,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PageRendererDefinition {
    ConventionFile,
    Extension {
        extension_type: ExtensionType,
        schema_version: u32,
        #[serde(default)]
        config: Value,
    },
}

impl Default for PageRendererDefinition {
    fn default() -> Self {
        Self::ConventionFile
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExtensionType(String);

impl ExtensionType {
    #[must_use]
    pub fn of<T: 'static>() -> Self {
        Self(std::any::type_name::<T>().to_owned())
    }

    #[must_use]
    pub fn from_provider_key(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn simple_name(&self) -> &str {
        self.0
            .rsplit_once("::")
            .map_or(self.0.as_str(), |(_, name)| name)
    }
}

impl std::fmt::Display for ExtensionType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AdminDefinition {
    #[must_use]
    pub fn page(&self, id: &DefinitionId) -> Option<&PageDefinition> {
        self.pages.iter().find(|page| &page.id == id)
    }

    #[must_use]
    pub fn scene(&self, id: &DefinitionId) -> Option<&SceneDefinition> {
        self.scenes.iter().find(|scene| &scene.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_type_is_derived_from_rust_type() {
        struct TestExtension;
        let extension_type = ExtensionType::of::<TestExtension>();
        assert!(extension_type.as_str().ends_with("::TestExtension"));
        assert_eq!(extension_type.simple_name(), "TestExtension");
    }
}
