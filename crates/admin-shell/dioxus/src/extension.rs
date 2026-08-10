use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Context, Result, bail, ensure};
use az_admin_shell_core::{CompiledPage, DefinitionId, ExtensionType, ResourceCatalog};
use dioxus::prelude::*;
use serde_json::Value;

use crate::AdminProviderHandle;

#[derive(Clone, PartialEq)]
pub struct PageExtensionEditorContext {
    pub page_id: DefinitionId,
    pub config: Value,
    pub resources: ResourceCatalog,
    pub on_change: Callback<Value>,
}

#[derive(Clone, PartialEq)]
pub struct PageExtensionRuntimeContext {
    pub page: CompiledPage,
    pub payload: Value,
    pub resources: ResourceCatalog,
    pub admin: AdminProviderHandle,
}

pub trait PageExtensionRenderer: Send + Sync + std::fmt::Debug {
    fn key(&self) -> &'static str;

    fn render_editor(&self, context: PageExtensionEditorContext) -> Element;

    fn render(&self, context: PageExtensionRuntimeContext) -> Element;
}

pub type DynPageExtensionRenderer = Arc<dyn PageExtensionRenderer>;

#[derive(Clone, Debug, Default)]
pub struct PageExtensionRendererIndex {
    providers: BTreeMap<ExtensionType, DynPageExtensionRenderer>,
}

impl PartialEq for PageExtensionRendererIndex {
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

impl PageExtensionRendererIndex {
    pub fn from_context(context: &mut rudi::Context) -> Result<Self> {
        let names = context
            .get_providers_by_type::<DynPageExtensionRenderer>()
            .into_iter()
            .map(|provider| provider.definition().key.name.to_string())
            .collect::<Vec<_>>();
        let mut providers = BTreeMap::new();
        for name in names {
            let provider = context
                .resolve_option_with_name::<DynPageExtensionRenderer>(name.clone())
                .with_context(|| format!("无法解析页面扩展渲染器: {name}"))?;
            ensure!(
                provider.key() == name,
                "页面扩展渲染器的 Rudi name 与 Provider key 不一致: {name} != {}",
                provider.key()
            );
            let extension_type = ExtensionType::from_provider_key(name);
            if providers.insert(extension_type.clone(), provider).is_some() {
                bail!("页面扩展渲染器重复: {extension_type}");
            }
        }
        Ok(Self { providers })
    }

    #[must_use]
    pub fn get(&self, extension_type: &ExtensionType) -> Option<&DynPageExtensionRenderer> {
        self.providers.get(extension_type)
    }
}

#[derive(Clone, PartialEq)]
pub struct ConventionPageContext {
    pub page: CompiledPage,
    pub admin: AdminProviderHandle,
}

pub trait ConventionPageProvider: Send + Sync + std::fmt::Debug {
    fn key(&self) -> &'static str;

    fn simple_name(&self) -> &'static str {
        let key = self.key();
        key.rsplit_once("::").map_or(key, |(_, name)| name)
    }

    fn render(&self, context: ConventionPageContext) -> Element;
}

pub type DynConventionPageProvider = Arc<dyn ConventionPageProvider>;

#[derive(Clone, Debug, Default)]
pub struct ConventionPageIndex {
    providers: BTreeMap<String, DynConventionPageProvider>,
}

impl PartialEq for ConventionPageIndex {
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

impl ConventionPageIndex {
    pub fn from_context(context: &mut rudi::Context) -> Result<Self> {
        let names = context
            .get_providers_by_type::<DynConventionPageProvider>()
            .into_iter()
            .map(|provider| provider.definition().key.name.to_string())
            .collect::<Vec<_>>();
        let mut providers = BTreeMap::new();
        for name in names {
            let provider = context
                .resolve_option_with_name::<DynConventionPageProvider>(name.clone())
                .with_context(|| format!("无法解析约定页面 Provider: {name}"))?;
            ensure!(
                provider.key() == name,
                "约定页面的 Rudi name 与 Provider key 不一致: {name} != {}",
                provider.key()
            );
            let simple_name = provider.simple_name().to_owned();
            if providers.insert(simple_name.clone(), provider).is_some() {
                bail!("约定页面模块名重复: {simple_name}");
            }
        }
        Ok(Self { providers })
    }

    #[must_use]
    pub fn get(&self, module_name: &str) -> Option<&DynConventionPageProvider> {
        self.providers.get(module_name)
    }
}
