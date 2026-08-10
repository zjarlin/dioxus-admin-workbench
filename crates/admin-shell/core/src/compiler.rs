use std::collections::{BTreeMap, BTreeSet};

use convert_case::{Case, Casing};
use deunicode::deunicode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AdminDefinition, DefinitionId, ExtensionType, PageCompileContext, PageExtensionCompilerIndex,
    PageRendererDefinition, ResourceCatalog,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<DefinitionId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompiledAdminDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub title: String,
    pub scenes: Vec<crate::SceneDefinition>,
    pub pages: BTreeMap<DefinitionId, CompiledPage>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompiledPage {
    pub id: DefinitionId,
    pub name: String,
    pub title: String,
    pub renderer: CompiledPageRenderer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompiledPageRenderer {
    ConventionFile {
        module_name: String,
        expected_path: String,
    },
    Extension {
        extension_type: ExtensionType,
        provider_key: String,
        schema_version: u32,
        payload: Value,
    },
}

pub struct AdminCompiler<'a> {
    extensions: &'a PageExtensionCompilerIndex,
    resources: &'a ResourceCatalog,
}

impl<'a> AdminCompiler<'a> {
    #[must_use]
    pub fn new(extensions: &'a PageExtensionCompilerIndex, resources: &'a ResourceCatalog) -> Self {
        Self {
            extensions,
            resources,
        }
    }

    pub fn compile(
        &self,
        definition: &AdminDefinition,
    ) -> Result<CompiledAdminDefinition, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        validate_definition(definition, &mut diagnostics);
        let mut pages = BTreeMap::new();
        for page in &definition.pages {
            match self.compile_page(definition, page) {
                Ok(compiled) => {
                    pages.insert(page.id.clone(), compiled);
                }
                Err(page_diagnostics) => diagnostics.extend(page_diagnostics),
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        Ok(CompiledAdminDefinition {
            id: definition.id.clone(),
            name: definition.name.clone(),
            title: definition.title.clone(),
            scenes: definition.scenes.clone(),
            pages,
        })
    }

    fn compile_page(
        &self,
        definition: &AdminDefinition,
        page: &crate::PageDefinition,
    ) -> Result<CompiledPage, Vec<Diagnostic>> {
        let renderer = match &page.renderer {
            PageRendererDefinition::ConventionFile => CompiledPageRenderer::ConventionFile {
                module_name: convention_page_module_name(&definition.name, &page.name),
                expected_path: convention_page_path(&definition.name, &page.name),
            },
            PageRendererDefinition::Extension {
                extension_type,
                schema_version,
                config,
            } => {
                let Some(provider) = self.extensions.get(extension_type) else {
                    return Err(vec![diagnostic(
                        "PAGE_EXTENSION_MISSING",
                        format!("页面扩展未注册: {extension_type}"),
                        &page.id,
                    )]);
                };
                if *schema_version != provider.schema_version() {
                    return Err(vec![diagnostic(
                        "PAGE_EXTENSION_SCHEMA_MISMATCH",
                        format!(
                            "页面扩展配置版本不匹配: {} != {}",
                            schema_version,
                            provider.schema_version()
                        ),
                        &page.id,
                    )]);
                }
                let context = PageCompileContext {
                    page_id: &page.id,
                    resources: self.resources,
                };
                let errors = provider.validate(context, config);
                if !errors.is_empty() {
                    return Err(errors
                        .into_iter()
                        .map(|message| diagnostic("PAGE_EXTENSION_INVALID", message, &page.id))
                        .collect());
                }
                let payload = provider.compile(context, config).map_err(|error| {
                    vec![diagnostic(
                        "PAGE_EXTENSION_COMPILE_FAILED",
                        error.to_string(),
                        &page.id,
                    )]
                })?;
                CompiledPageRenderer::Extension {
                    extension_type: extension_type.clone(),
                    provider_key: provider.key().to_owned(),
                    schema_version: *schema_version,
                    payload,
                }
            }
        };
        Ok(CompiledPage {
            id: page.id.clone(),
            name: page.name.clone(),
            title: page.title.clone(),
            renderer,
        })
    }
}

fn validate_definition(definition: &AdminDefinition, diagnostics: &mut Vec<Diagnostic>) {
    let page_ids = definition
        .pages
        .iter()
        .map(|page| page.id.clone())
        .collect::<BTreeSet<_>>();
    let mut identities = BTreeSet::new();
    for page in &definition.pages {
        if !identities.insert(("page", page.id.clone())) {
            diagnostics.push(diagnostic(
                "PAGE_ID_DUPLICATE",
                format!("页面 ID 重复: {}", page.id),
                &page.id,
            ));
        }
    }
    for scene in &definition.scenes {
        validate_menus(&scene.menus, &page_ids, &mut identities, diagnostics);
    }
}

fn validate_menus(
    menus: &[crate::MenuDefinition],
    page_ids: &BTreeSet<DefinitionId>,
    identities: &mut BTreeSet<(&'static str, DefinitionId)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for menu in menus {
        if !identities.insert(("menu", menu.id.clone())) {
            diagnostics.push(diagnostic(
                "MENU_ID_DUPLICATE",
                format!("菜单 ID 重复: {}", menu.id),
                &menu.id,
            ));
        }
        if let Some(page_id) = &menu.page_id
            && !page_ids.contains(page_id)
        {
            diagnostics.push(diagnostic(
                "MENU_PAGE_MISSING",
                format!("菜单引用的页面不存在: {page_id}"),
                &menu.id,
            ));
        }
        validate_menus(&menu.children, page_ids, identities, diagnostics);
    }
}

fn diagnostic(code: &str, message: String, target_id: &DefinitionId) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        message,
        target_id: Some(target_id.clone()),
    }
}

#[must_use]
pub fn convention_page_module_name(program_name: &str, page_name: &str) -> String {
    format!(
        "{}__{}",
        identifier(program_name, "program"),
        identifier(page_name, "page")
    )
}

#[must_use]
pub fn convention_page_path(program_name: &str, page_name: &str) -> String {
    format!(
        "src/pages/{}.rs",
        convention_page_module_name(program_name, page_name)
    )
}

fn identifier(value: &str, fallback: &str) -> String {
    let normalized = deunicode(value).to_case(Case::Snake);
    let normalized = normalized
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    let normalized = normalized.trim_matches('_');
    if normalized.is_empty() {
        return fallback.to_owned();
    }
    if normalized.starts_with(|character: char| character.is_ascii_digit()) {
        return format!("_{normalized}");
    }
    normalized.to_owned()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use serde_json::json;

    use super::*;
    use crate::{
        AdminDefinition, PageDefinition, PageExtensionCompiler, ResourceDefinition,
        ResourceOperations,
    };

    #[derive(Debug)]
    struct TestExtension;

    impl PageExtensionCompiler for TestExtension {
        fn key(&self) -> &'static str {
            std::any::type_name::<Self>()
        }

        fn title(&self) -> &'static str {
            "测试"
        }

        fn description(&self) -> &'static str {
            "测试扩展"
        }

        fn schema_version(&self) -> u32 {
            1
        }

        fn default_config(&self) -> Value {
            json!({"resource_id": "users"})
        }

        fn validate(&self, context: PageCompileContext<'_>, config: &Value) -> Vec<String> {
            let Some(resource_id) = config.get("resource_id").and_then(Value::as_str) else {
                return vec!["缺少 resource_id".to_owned()];
            };
            if context.resources.get(resource_id).is_none() {
                return vec![format!("资源不存在: {resource_id}")];
            }
            Vec::new()
        }

        fn compile(&self, _context: PageCompileContext<'_>, config: &Value) -> Result<Value> {
            Ok(config.clone())
        }
    }

    #[test]
    fn registered_extension_compiles_to_provider_instruction() -> Result<()> {
        let provider: crate::DynPageExtensionCompiler = Arc::new(TestExtension);
        let index = PageExtensionCompilerIndex::from_providers(vec![provider])?;
        let mut resources = ResourceCatalog::default();
        resources.resources.insert(
            "users".to_owned(),
            ResourceDefinition {
                id: "users".to_owned(),
                name: "users".to_owned(),
                title: "用户".to_owned(),
                id_field: "id".to_owned(),
                fields: Vec::new(),
                operations: ResourceOperations::default(),
            },
        );
        let page_id = DefinitionId::new();
        let definition = AdminDefinition {
            id: DefinitionId::new(),
            name: "demo".to_owned(),
            title: "Demo".to_owned(),
            scenes: Vec::new(),
            pages: vec![PageDefinition {
                id: page_id.clone(),
                name: "users".to_owned(),
                title: "用户".to_owned(),
                renderer: PageRendererDefinition::Extension {
                    extension_type: ExtensionType::of::<TestExtension>(),
                    schema_version: 1,
                    config: json!({"resource_id": "users"}),
                },
            }],
        };

        let compiled = AdminCompiler::new(&index, &resources)
            .compile(&definition)
            .map_err(|diagnostics| anyhow::anyhow!("{diagnostics:?}"))?;
        let renderer = &compiled
            .pages
            .get(&page_id)
            .ok_or_else(|| anyhow::anyhow!("页面应存在"))?
            .renderer;
        assert!(matches!(renderer, CompiledPageRenderer::Extension { .. }));
        Ok(())
    }

    #[test]
    fn convention_path_is_deterministic() {
        assert_eq!(
            convention_page_path("AIO First Party", "API Keys"),
            "src/pages/aio_first_party__api_keys.rs"
        );
    }
}
