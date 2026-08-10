use std::{pin::Pin, sync::Arc};

use anyhow::{Context, Result, bail, ensure};
use az_admin_shell_core::{
    AdminCommand, AdminDefinition, CompiledAdminDefinition, DefinitionId, ResourceCatalog,
    ResourceRequest, ResourceResponse,
};

pub type AdminFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + 'static>>;

#[derive(Clone, Debug, PartialEq)]
pub struct AdminSnapshot {
    pub definition: AdminDefinition,
    pub compiled: CompiledAdminDefinition,
    pub resources: ResourceCatalog,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConventionFileResult {
    pub path: String,
    pub created: bool,
}

pub trait AdminProvider: Send + Sync + std::fmt::Debug {
    fn key(&self) -> &'static str;

    fn load(&self) -> AdminFuture<AdminSnapshot>;

    fn execute(&self, command: AdminCommand) -> AdminFuture<AdminSnapshot>;

    fn execute_resource(&self, request: ResourceRequest) -> AdminFuture<ResourceResponse>;

    fn generate_convention_file(&self, page_id: DefinitionId) -> AdminFuture<ConventionFileResult>;
}

pub type DynAdminProvider = Arc<dyn AdminProvider>;

#[derive(Clone)]
pub struct AdminProviderHandle(DynAdminProvider);

impl AdminProviderHandle {
    #[must_use]
    pub fn new(provider: DynAdminProvider) -> Self {
        Self(provider)
    }

    #[must_use]
    pub fn provider(&self) -> &DynAdminProvider {
        &self.0
    }
}

impl PartialEq for AdminProviderHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for AdminProviderHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("AdminProviderHandle")
            .field(&self.0.key())
            .finish()
    }
}

pub fn resolve_admin_provider(context: &mut rudi::Context) -> Result<AdminProviderHandle> {
    let names = context
        .get_providers_by_type::<DynAdminProvider>()
        .into_iter()
        .map(|provider| provider.definition().key.name.to_string())
        .collect::<Vec<_>>();
    if names.is_empty() {
        bail!("未注册 AdminProvider");
    }
    ensure!(
        names.len() == 1,
        "AdminProvider 必须且只能注册一个: {names:?}"
    );
    let name = names[0].clone();
    let provider = context
        .resolve_option_with_name::<DynAdminProvider>(name.clone())
        .with_context(|| format!("无法解析 AdminProvider: {name}"))?;
    ensure!(
        provider.key() == name,
        "AdminProvider 的 Rudi name 与 Provider key 不一致: {name} != {}",
        provider.key()
    );
    Ok(AdminProviderHandle::new(provider))
}
