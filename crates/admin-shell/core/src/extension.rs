use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

use crate::{DefinitionId, ExtensionType, ResourceCatalog};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageExtensionDescriptor {
    pub extension_type: ExtensionType,
    pub title: String,
    pub description: String,
    pub schema_version: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PageCompileContext<'a> {
    pub page_id: &'a DefinitionId,
    pub resources: &'a ResourceCatalog,
}

pub trait PageExtensionCompiler: Send + Sync + std::fmt::Debug {
    fn key(&self) -> &'static str;

    fn title(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn schema_version(&self) -> u32;

    fn default_config(&self) -> Value;

    fn validate(&self, context: PageCompileContext<'_>, config: &Value) -> Vec<String>;

    fn compile(&self, context: PageCompileContext<'_>, config: &Value) -> Result<Value>;

    fn descriptor(&self) -> PageExtensionDescriptor {
        PageExtensionDescriptor {
            extension_type: ExtensionType::from_provider_key(self.key()),
            title: self.title().to_owned(),
            description: self.description().to_owned(),
            schema_version: self.schema_version(),
        }
    }
}

pub type DynPageExtensionCompiler = Arc<dyn PageExtensionCompiler>;

#[derive(Clone, Debug, Default)]
pub struct PageExtensionCompilerIndex {
    providers: BTreeMap<ExtensionType, DynPageExtensionCompiler>,
}

impl PartialEq for PageExtensionCompilerIndex {
    fn eq(&self, other: &Self) -> bool {
        self.providers.len() == other.providers.len()
            && self.providers.iter().all(|(key, provider)| {
                other
                    .providers
                    .get(key)
                    .is_some_and(|other| Arc::ptr_eq(provider, other))
            })
    }
}

impl PageExtensionCompilerIndex {
    pub fn from_context(context: &mut rudi::Context) -> Result<Self> {
        let names = context
            .get_providers_by_type::<DynPageExtensionCompiler>()
            .into_iter()
            .map(|provider| provider.definition().key.name.to_string())
            .collect::<Vec<_>>();
        let mut providers = Vec::with_capacity(names.len());
        for name in names {
            let provider = context
                .resolve_option_with_name::<DynPageExtensionCompiler>(name.clone())
                .with_context(|| format!("无法解析页面扩展编译器: {name}"))?;
            ensure!(
                provider.key() == name,
                "页面扩展的 Rudi name 与 Provider key 不一致: {name} != {}",
                provider.key()
            );
            providers.push(provider);
        }
        Self::from_providers(providers)
    }

    pub fn from_providers(providers: Vec<DynPageExtensionCompiler>) -> Result<Self> {
        let mut index = BTreeMap::new();
        for provider in providers {
            let extension_type = ExtensionType::from_provider_key(provider.key());
            if index.insert(extension_type.clone(), provider).is_some() {
                bail!("页面扩展编译器重复: {extension_type}");
            }
        }
        Ok(Self { providers: index })
    }

    #[must_use]
    pub fn get(&self, extension_type: &ExtensionType) -> Option<&DynPageExtensionCompiler> {
        self.providers.get(extension_type)
    }

    #[must_use]
    pub fn descriptors(&self) -> Vec<PageExtensionDescriptor> {
        self.providers
            .values()
            .map(|provider| provider.descriptor())
            .collect()
    }
}
