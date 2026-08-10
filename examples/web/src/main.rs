use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result, bail};
use az_admin_shell_core::{
    AdminCommand, AdminCompiler, AdminDefinition, DefinitionId, ExtensionType, MenuDefinition,
    PageDefinition, PageExtensionCompilerIndex, PageRendererDefinition, ResourceCatalog,
    ResourceDefinition, ResourceFieldDefinition, ResourceFieldKind, ResourceOperations,
    ResourcePage, ResourceRecord, ResourceRequest, ResourceResponse, SceneDefinition,
};
use az_dioxus_admin_extension_crud::{CrudPageConfig, CrudPageExtension};
use az_dioxus_admin_shell::{
    AdminFuture, AdminProvider, AdminSnapshot, ConventionFileResult, ConventionPageContext,
    ConventionPageProvider, DynAdminProvider, DynConventionPageProvider,
};
use dioxus::prelude::*;
use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct DemoProvider {
    state: Arc<Mutex<DemoState>>,
}

#[derive(Debug)]
struct DemoState {
    definition: AdminDefinition,
    resources: ResourceCatalog,
    records: BTreeMap<String, Vec<ResourceRecord>>,
}

impl DemoProvider {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(demo_state())),
        }
    }

    fn snapshot(&self) -> Result<AdminSnapshot> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("示例状态锁已损坏"))?;
        let mut context = rudi::Context::auto_register();
        let extensions = PageExtensionCompilerIndex::from_context(&mut context)?;
        let compiled = AdminCompiler::new(&extensions, &state.resources)
            .compile(&state.definition)
            .map_err(|diagnostics| anyhow::anyhow!("编译工作台失败: {diagnostics:?}"))?;
        Ok(AdminSnapshot {
            definition: state.definition.clone(),
            compiled,
            resources: state.resources.clone(),
        })
    }
}

impl AdminProvider for DemoProvider {
    fn key(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn load(&self) -> AdminFuture<AdminSnapshot> {
        let provider = self.clone();
        Box::pin(async move { provider.snapshot() })
    }

    fn execute(&self, command: AdminCommand) -> AdminFuture<AdminSnapshot> {
        let provider = self.clone();
        Box::pin(async move {
            {
                let mut state = provider
                    .state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("示例状态锁已损坏"))?;
                state.definition.apply(command)?;
            }
            provider.snapshot()
        })
    }

    fn execute_resource(&self, request: ResourceRequest) -> AdminFuture<ResourceResponse> {
        let provider = self.clone();
        Box::pin(async move {
            let mut state = provider
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("示例状态锁已损坏"))?;
            execute_resource(&mut state, request)
        })
    }

    fn generate_convention_file(&self, page_id: DefinitionId) -> AdminFuture<ConventionFileResult> {
        let provider = self.clone();
        Box::pin(async move {
            let state = provider
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("示例状态锁已损坏"))?;
            let page = state
                .definition
                .page(&page_id)
                .with_context(|| format!("页面不存在: {page_id}"))?;
            Ok(ConventionFileResult {
                path: az_admin_shell_core::convention_page_path(&state.definition.name, &page.name),
                created: false,
            })
        })
    }
}

#[rudi::Singleton(name = std::any::type_name::<DemoProvider>())]
fn admin_provider() -> DynAdminProvider {
    Arc::new(DemoProvider::new())
}

#[allow(non_snake_case)]
mod demo__welcome {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct WelcomePage;

    impl ConventionPageProvider for WelcomePage {
        fn key(&self) -> &'static str {
            module_path!()
        }

        fn render(&self, context: ConventionPageContext) -> Element {
            rsx! {
                section {
                    h2 { "{context.page.title}" }
                    p { "这是消费方通过约定文件注册的页面。" }
                }
            }
        }
    }

    #[rudi::Singleton(name = module_path!())]
    fn convention_page() -> DynConventionPageProvider {
        Arc::new(WelcomePage)
    }
}

fn demo_state() -> DemoState {
    let welcome_page_id = DefinitionId::new();
    let users_page_id = DefinitionId::new();
    let users_resource = ResourceDefinition {
        id: "users".to_owned(),
        name: "users".to_owned(),
        title: "用户".to_owned(),
        id_field: "id".to_owned(),
        fields: vec![
            field("id", "ID", ResourceFieldKind::Text, true),
            field("name", "姓名", ResourceFieldKind::Text, true),
            field("email", "邮箱", ResourceFieldKind::Text, true),
            field("enabled", "启用", ResourceFieldKind::Boolean, false),
        ],
        operations: ResourceOperations {
            list: true,
            create: true,
            update: true,
            delete: true,
        },
    };
    let definition = AdminDefinition {
        id: DefinitionId::new(),
        name: "demo".to_owned(),
        title: "Admin Workbench".to_owned(),
        scenes: vec![SceneDefinition {
            id: DefinitionId::new(),
            name: "workspace".to_owned(),
            title: "工作区".to_owned(),
            menus: vec![
                menu("welcome", "欢迎", &welcome_page_id),
                menu("users", "用户", &users_page_id),
            ],
        }],
        pages: vec![
            PageDefinition {
                id: welcome_page_id,
                name: "welcome".to_owned(),
                title: "欢迎".to_owned(),
                renderer: PageRendererDefinition::ConventionFile,
            },
            PageDefinition {
                id: users_page_id,
                name: "users".to_owned(),
                title: "用户".to_owned(),
                renderer: PageRendererDefinition::Extension {
                    extension_type: ExtensionType::of::<CrudPageExtension>(),
                    schema_version: 1,
                    config: serde_json::to_value(CrudPageConfig {
                        resource_id: "users".to_owned(),
                        page_size: 20,
                    })
                    .unwrap_or_default(),
                },
            },
        ],
    };
    let records = BTreeMap::from([(
        "users".to_owned(),
        vec![
            record("1", "Ada", "ada@example.com", true),
            record("2", "Linus", "linus@example.com", true),
        ],
    )]);
    DemoState {
        definition,
        resources: ResourceCatalog {
            resources: BTreeMap::from([("users".to_owned(), users_resource)]),
        },
        records,
    }
}

fn field(
    name: &str,
    title: &str,
    kind: ResourceFieldKind,
    required: bool,
) -> ResourceFieldDefinition {
    ResourceFieldDefinition {
        name: name.to_owned(),
        title: title.to_owned(),
        kind,
        required,
        list_visible: true,
        form_visible: name != "id",
    }
}

fn menu(name: &str, title: &str, page_id: &DefinitionId) -> MenuDefinition {
    MenuDefinition {
        id: DefinitionId::new(),
        name: name.to_owned(),
        title: title.to_owned(),
        icon: None,
        page_id: Some(page_id.clone()),
        enabled: true,
        children: Vec::new(),
    }
}

fn record(id: &str, name: &str, email: &str, enabled: bool) -> ResourceRecord {
    BTreeMap::from([
        ("id".to_owned(), json!(id)),
        ("name".to_owned(), json!(name)),
        ("email".to_owned(), json!(email)),
        ("enabled".to_owned(), json!(enabled)),
    ])
}

fn execute_resource(state: &mut DemoState, request: ResourceRequest) -> Result<ResourceResponse> {
    match request {
        ResourceRequest::List {
            resource_id,
            page,
            page_size,
        } => {
            let records = state.records.entry(resource_id).or_default();
            let start = page.saturating_mul(page_size) as usize;
            let items = records
                .iter()
                .skip(start)
                .take(page_size as usize)
                .cloned()
                .collect();
            Ok(ResourceResponse::Page(ResourcePage {
                items,
                total: records.len() as u64,
            }))
        }
        ResourceRequest::Create {
            resource_id,
            mut values,
        } => {
            values
                .entry("id".to_owned())
                .or_insert_with(|| Value::String(uuid::Uuid::new_v4().to_string()));
            state
                .records
                .entry(resource_id)
                .or_default()
                .push(values.clone());
            Ok(ResourceResponse::Record(values))
        }
        ResourceRequest::Update {
            resource_id,
            record_id,
            values,
        } => {
            let records = state.records.entry(resource_id).or_default();
            let record = records
                .iter_mut()
                .find(|record| record.get("id").and_then(Value::as_str) == Some(&record_id))
                .with_context(|| format!("记录不存在: {record_id}"))?;
            record.extend(values);
            Ok(ResourceResponse::Record(record.clone()))
        }
        ResourceRequest::Delete {
            resource_id,
            record_id,
        } => {
            let records = state.records.entry(resource_id).or_default();
            let previous = records.len();
            records.retain(|record| record.get("id").and_then(Value::as_str) != Some(&record_id));
            if previous == records.len() {
                bail!("记录不存在: {record_id}");
            }
            Ok(ResourceResponse::Deleted)
        }
    }
}

fn main() {
    az_dioxus_admin_extension_crud::enable();
    dioxus::launch(az_dioxus_admin_shell::App);
}

rudi::enable! {}
