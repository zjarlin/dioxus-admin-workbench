use anyhow::{Context, Result};
use az_admin_shell_core::{PageCompileContext, PageExtensionCompiler};
use serde_json::Value;

use crate::{CrudPageConfig, CrudPageExtension};

impl PageExtensionCompiler for CrudPageExtension {
    fn key(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn title(&self) -> &'static str {
        "资源增删改查"
    }

    fn description(&self) -> &'static str {
        "根据资源字段和操作契约生成 DataTable 与 Dialog 表单"
    }

    fn schema_version(&self) -> u32 {
        1
    }

    fn default_config(&self) -> Value {
        serde_json::to_value(CrudPageConfig::default()).unwrap_or_default()
    }

    fn validate(&self, context: PageCompileContext<'_>, config: &Value) -> Vec<String> {
        let config = match serde_json::from_value::<CrudPageConfig>(config.clone()) {
            Ok(config) => config,
            Err(error) => return vec![format!("CRUD 配置无效: {error}")],
        };
        if config.resource_id.trim().is_empty() {
            return vec!["CRUD 必须选择资源".to_owned()];
        }
        let Some(resource) = context.resources.get(&config.resource_id) else {
            return vec![format!("CRUD 资源不存在: {}", config.resource_id)];
        };
        if !resource.operations.list {
            return vec![format!("资源不支持列表操作: {}", resource.title)];
        }
        if !(1..=200).contains(&config.page_size) {
            return vec!["CRUD 每页条数必须在 1..=200".to_owned()];
        }
        Vec::new()
    }

    fn compile(&self, _context: PageCompileContext<'_>, config: &Value) -> Result<Value> {
        let config = serde_json::from_value::<CrudPageConfig>(config.clone())
            .context("解析 CRUD 页面配置失败")?;
        serde_json::to_value(config).context("编译 CRUD 页面配置失败")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use az_admin_shell_core::{
        DefinitionId, ResourceCatalog, ResourceDefinition, ResourceOperations,
    };
    use serde_json::json;

    use super::*;

    #[test]
    fn crud_requires_listable_resource() {
        let provider = CrudPageExtension;
        let resources = ResourceCatalog {
            resources: BTreeMap::from([(
                "users".to_owned(),
                ResourceDefinition {
                    id: "users".to_owned(),
                    name: "users".to_owned(),
                    title: "用户".to_owned(),
                    id_field: "id".to_owned(),
                    fields: Vec::new(),
                    operations: ResourceOperations::default(),
                },
            )]),
        };
        let page_id = DefinitionId::new();
        let errors = provider.validate(
            PageCompileContext {
                page_id: &page_id,
                resources: &resources,
            },
            &json!({"resource_id": "users", "page_size": 20}),
        );
        assert_eq!(errors, vec!["资源不支持列表操作: 用户"]);
    }
}
