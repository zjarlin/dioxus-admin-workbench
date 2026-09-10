use std::{
    any::{Any, TypeId},
    collections::HashSet,
    sync::Arc,
};

use anyhow::{Context as _, Result, ensure};
use dill::{AllOf, Catalog};
use dioxus::prelude::Element;

use crate::{
    ApplicationAccountItem, ApplicationMenuGroup,
    navigation_validation::{NavigationContribution, validate_navigation_contributions},
};

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
    pub menu_path: Vec<ApplicationMenuGroup>,
    pub required_permission: Option<&'static str>,
    pub render: fn() -> Element,
}

impl PartialEq for ApplicationPage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.icon == other.icon
            && self.scene == other.scene
            && self.menu_path == other.menu_path
            && self.required_permission == other.required_permission
            && std::ptr::fn_addr_eq(self.render, other.render)
    }
}

/// 可被 Dill 聚合的页面插件；具体 Rust 类型是唯一运行时身份。
pub trait ApplicationPlugin: Any + Send + Sync {
    fn pages(&self) -> Vec<ApplicationPage>;
}

/// 账户插件向左下角账户区贡献动作或页面。
pub trait ApplicationAccountPlugin: Any + Send + Sync {
    fn items(&self) -> Vec<ApplicationAccountItem>;
}

pub type DynApplicationPlugin = Arc<dyn ApplicationPlugin>;

/// 从 Dill 收集插件并校验页面导航契约。
pub fn collect_application_pages(catalog: &Catalog) -> Result<Vec<ApplicationPage>> {
    let plugins = catalog
        .get::<AllOf<dyn ApplicationPlugin>>()
        .context("从 Dill 聚合页面插件失败")?;
    let mut plugin_types = HashSet::<TypeId>::new();
    let mut pages = Vec::new();

    for plugin in plugins {
        let plugin_type = plugin.as_ref().type_id();
        ensure!(
            plugin_types.insert(plugin_type),
            "同一页面插件类型被重复注册: {plugin_type:?}"
        );
        for page in plugin.pages() {
            pages.push(page);
        }
    }

    ensure!(!pages.is_empty(), "应用至少需要一个页面");
    validate_pages(&pages)?;
    Ok(pages)
}

/// 从 Dill 聚合账户插件，账户动作身份只在当前组合内唯一。
pub fn collect_application_account_items(catalog: &Catalog) -> Result<Vec<ApplicationAccountItem>> {
    let plugins = catalog
        .get::<AllOf<dyn ApplicationAccountPlugin>>()
        .context("从 Dill 聚合账户插件失败")?;
    let mut ids = HashSet::<String>::new();
    let mut items = Vec::new();
    for plugin in plugins {
        for item in plugin.items() {
            ensure!(!item.id.trim().is_empty(), "账户动作 id 不能为空");
            ensure!(!item.label.trim().is_empty(), "账户动作标题不能为空");
            ensure!(
                item.required_permission
                    .as_deref()
                    .is_none_or(|permission| !permission.trim().is_empty()),
                "账户动作权限不能为空字符串"
            );
            ensure!(ids.insert(item.id.clone()), "账户动作 id 重复: {}", item.id);
            items.push(item);
        }
    }
    Ok(items)
}

fn validate_pages(pages: &[ApplicationPage]) -> Result<()> {
    for page in pages {
        ensure!(
            page.required_permission
                .is_none_or(|permission| !permission.trim().is_empty()),
            "页面权限不能为空字符串"
        );
    }
    validate_navigation_contributions(pages.iter().map(|page| NavigationContribution {
        page_id: page.id,
        page_label: page.label,
        scene_id: page.scene.id,
        scene_label: page.scene.label,
        menu_path: &page.menu_path,
        sidebar: true,
    }))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use dioxus::prelude::*;

    use super::*;

    struct FirstPlugin;
    struct DuplicatePagePlugin;
    struct AccountPlugin;

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

    impl ApplicationAccountPlugin for AccountPlugin {
        fn items(&self) -> Vec<ApplicationAccountItem> {
            vec![ApplicationAccountItem {
                id: "profile".to_owned(),
                label: "个人资料".to_owned(),
                icon: Some("user".to_owned()),
                page_id: Some("profile".to_owned()),
                required_permission: None,
                destructive: false,
            }]
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
            menu_path: Vec::new(),
            required_permission: None,
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

    #[test]
    fn collects_account_items_from_dill() -> Result<()> {
        let catalog = Catalog::builder()
            .add_value(AccountPlugin)
            .bind::<dyn ApplicationAccountPlugin, AccountPlugin>()
            .build();

        let items = collect_application_account_items(&catalog)?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].page_id.as_deref(), Some("profile"));
        Ok(())
    }

    struct ConflictingGroupPlugin;

    impl ApplicationPlugin for ConflictingGroupPlugin {
        fn pages(&self) -> Vec<ApplicationPage> {
            let mut users = page("users");
            users.menu_path = vec![group("system-management", "系统管理")];
            let mut roles = page("roles");
            roles.menu_path = vec![group("system-management", "基础设施")];
            vec![users, roles]
        }
    }

    struct CyclicGroupPlugin;

    impl ApplicationPlugin for CyclicGroupPlugin {
        fn pages(&self) -> Vec<ApplicationPage> {
            let mut page = page("dictionary");
            page.menu_path = vec![
                group("system-management", "系统管理"),
                group("system-management", "系统管理"),
            ];
            vec![page]
        }
    }

    struct ConflictingParentPlugin;

    impl ApplicationPlugin for ConflictingParentPlugin {
        fn pages(&self) -> Vec<ApplicationPage> {
            let mut dictionary = page("dictionary");
            dictionary.menu_path = vec![
                group("system-management", "系统管理"),
                group("shared-tools", "共享工具"),
            ];
            let mut files = page("files");
            files.menu_path = vec![
                group("infrastructure", "基础设施"),
                group("shared-tools", "共享工具"),
            ];
            vec![dictionary, files]
        }
    }

    fn group(id: &str, label: &str) -> ApplicationMenuGroup {
        ApplicationMenuGroup {
            id: id.to_owned(),
            label: label.to_owned(),
            icon: None,
        }
    }

    #[test]
    fn rejects_conflicting_group_contributions() -> Result<()> {
        let catalog = Catalog::builder()
            .add_value(ConflictingGroupPlugin)
            .bind::<dyn ApplicationPlugin, ConflictingGroupPlugin>()
            .build();

        let error = collect_application_pages(&catalog)
            .err()
            .context("冲突的菜单分组必须被拒绝")?;

        assert!(error.to_string().contains("展示信息不一致"));
        Ok(())
    }

    #[test]
    fn rejects_cycles_in_group_paths() -> Result<()> {
        let catalog = Catalog::builder()
            .add_value(CyclicGroupPlugin)
            .bind::<dyn ApplicationPlugin, CyclicGroupPlugin>()
            .build();

        let error = collect_application_pages(&catalog)
            .err()
            .context("循环菜单路径必须被拒绝")?;

        assert!(error.to_string().contains("形成循环"));
        Ok(())
    }

    #[test]
    fn rejects_the_same_group_under_different_parents() -> Result<()> {
        let catalog = Catalog::builder()
            .add_value(ConflictingParentPlugin)
            .bind::<dyn ApplicationPlugin, ConflictingParentPlugin>()
            .build();

        let error = collect_application_pages(&catalog)
            .err()
            .context("同一分组不能挂到两个父节点")?;

        assert!(error.to_string().contains("父节点"));
        Ok(())
    }
}
