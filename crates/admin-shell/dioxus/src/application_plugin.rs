use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::{Context as _, Result, ensure};
use dill::{AllOf, Catalog};
use dioxus::prelude::Element;

/// 壳层中的业务场景。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationScene {
    pub id: &'static str,
    pub label: &'static str,
}

/// 插件向壳层贡献的业务页面。
#[derive(Clone, Debug)]
pub struct ApplicationPage {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: Option<&'static str>,
    pub scene: ApplicationScene,
    pub render: fn() -> Element,
}

impl PartialEq for ApplicationPage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.icon == other.icon
            && self.scene == other.scene
            && std::ptr::fn_addr_eq(self.render, other.render)
    }
}

/// 可被 Dill 聚合的页面插件；具体 Rust 类型是唯一运行时身份。
pub trait ApplicationPlugin: Any + Send + Sync {
    fn pages(&self) -> Vec<ApplicationPage>;
}

pub type DynApplicationPlugin = Arc<dyn ApplicationPlugin>;

/// 从 Dill 收集插件并校验页面导航契约。
pub fn collect_application_pages(catalog: &Catalog) -> Result<Vec<ApplicationPage>> {
    let plugins = catalog
        .get::<AllOf<dyn ApplicationPlugin>>()
        .context("从 Dill 聚合页面插件失败")?;
    let mut plugin_types = HashSet::<TypeId>::new();
    let mut page_ids = HashSet::<&'static str>::new();
    let mut scenes = HashMap::<&'static str, &'static str>::new();
    let mut pages = Vec::new();

    for plugin in plugins {
        let plugin_type = plugin.as_ref().type_id();
        ensure!(
            plugin_types.insert(plugin_type),
            "同一页面插件类型被重复注册: {plugin_type:?}"
        );
        for page in plugin.pages() {
            validate_page(&page, &mut page_ids, &mut scenes)?;
            pages.push(page);
        }
    }

    ensure!(!pages.is_empty(), "应用至少需要一个页面");
    Ok(pages)
}

fn validate_page(
    page: &ApplicationPage,
    page_ids: &mut HashSet<&'static str>,
    scenes: &mut HashMap<&'static str, &'static str>,
) -> Result<()> {
    ensure!(!page.id.trim().is_empty(), "页面 id 不能为空");
    ensure!(!page.label.trim().is_empty(), "页面标题不能为空");
    ensure!(!page.scene.id.trim().is_empty(), "场景 id 不能为空");
    ensure!(!page.scene.label.trim().is_empty(), "场景标题不能为空");
    ensure!(page_ids.insert(page.id), "页面 id 重复: {}", page.id);

    if let Some(label) = scenes.insert(page.scene.id, page.scene.label) {
        ensure!(
            label == page.scene.label,
            "同一场景 id 的标题不一致: {}",
            page.scene.id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use dioxus::prelude::*;

    use super::*;

    struct FirstPlugin;
    struct DuplicatePagePlugin;

    impl ApplicationPlugin for FirstPlugin {
        fn pages(&self) -> Vec<ApplicationPage> {
            vec![page("home")]
        }
    }

    impl ApplicationPlugin for DuplicatePagePlugin {
        fn pages(&self) -> Vec<ApplicationPage> {
            vec![page("home")]
        }
    }

    fn page(id: &'static str) -> ApplicationPage {
        ApplicationPage {
            id,
            label: "首页",
            icon: None,
            scene: ApplicationScene {
                id: "workspace",
                label: "工作区",
            },
            render: render_page,
        }
    }

    fn render_page() -> Element {
        rsx! { p { "页面" } }
    }

    #[test]
    fn collects_pages_from_dill_by_concrete_type() -> Result<()> {
        let catalog = Catalog::builder()
            .add_value(FirstPlugin)
            .bind::<dyn ApplicationPlugin, FirstPlugin>()
            .build();

        let pages = collect_application_pages(&catalog)?;

        assert_eq!(pages, vec![page("home")]);
        Ok(())
    }

    #[test]
    fn rejects_duplicate_page_ids_across_plugins() -> Result<()> {
        let catalog = Catalog::builder()
            .add_value(FirstPlugin)
            .bind::<dyn ApplicationPlugin, FirstPlugin>()
            .add_value(DuplicatePagePlugin)
            .bind::<dyn ApplicationPlugin, DuplicatePagePlugin>()
            .build();

        let error = collect_application_pages(&catalog)
            .err()
            .context("重复页面必须被拒绝")?;

        assert!(error.to_string().contains("页面 id 重复"));
        Ok(())
    }
}
