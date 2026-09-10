use std::collections::HashSet;

use anyhow::Result;

use crate::{
    ApplicationAccountItem, ApplicationMenuGroup, ApplicationMenuItem, ApplicationPage,
    ApplicationRuntimePage, ApplicationSceneItem,
    navigation_validation::{NavigationContribution, validate_navigation_contributions},
};

pub(crate) struct PluginNavigation<'a> {
    pages: Vec<NavigationPage<'a>>,
}

struct NavigationPage<'a> {
    id: &'a str,
    label: &'a str,
    icon: Option<&'a str>,
    scene_id: &'a str,
    scene_label: &'a str,
    menu_path: &'a [ApplicationMenuGroup],
    account: bool,
}

impl<'a> PluginNavigation<'a> {
    pub(crate) fn new(
        pages: &'a [ApplicationPage],
        runtime_pages: &'a [ApplicationRuntimePage],
        account_items: &[ApplicationAccountItem],
    ) -> Result<Self> {
        let account_pages = account_items
            .iter()
            .filter_map(|item| item.page_id.as_deref())
            .collect::<HashSet<_>>();
        let pages = pages
            .iter()
            .map(|page| NavigationPage {
                id: page.id,
                label: page.label,
                icon: page.icon,
                scene_id: page.scene.id,
                scene_label: page.scene.label,
                menu_path: &page.menu_path,
                account: account_pages.contains(page.id),
            })
            .chain(runtime_pages.iter().map(|page| NavigationPage {
                id: &page.id,
                label: &page.label,
                icon: page.icon.as_deref(),
                scene_id: &page.scene_id,
                scene_label: &page.scene_label,
                menu_path: &page.menu_path,
                account: account_pages.contains(page.id.as_str()),
            }))
            .collect::<Vec<_>>();
        validate_navigation_contributions(pages.iter().map(|page| NavigationContribution {
            page_id: page.id,
            page_label: page.label,
            scene_id: page.scene_id,
            scene_label: page.scene_label,
            menu_path: page.menu_path,
            sidebar: !page.account,
        }))?;
        Ok(Self { pages })
    }

    pub(crate) fn workspace_page(&self, selected: Option<&str>) -> Option<&'a str> {
        self.pages
            .iter()
            .find(|page| !page.account && Some(page.id) == selected)
            .or_else(|| self.pages.iter().find(|page| !page.account))
            .map(|page| page.id)
    }

    pub(crate) fn account_page(&self, selected: Option<&str>) -> Option<&'a str> {
        self.pages
            .iter()
            .find(|page| page.account && Some(page.id) == selected)
            .map(|page| page.id)
    }

    pub(crate) fn scene_for_page(&self, selected: Option<&str>) -> Option<&'a str> {
        self.pages
            .iter()
            .find(|page| !page.account && Some(page.id) == selected)
            .map(|page| page.scene_id)
    }

    pub(crate) fn first_page_in_scene(&self, scene_id: &str) -> Option<&'a str> {
        self.pages
            .iter()
            .find(|page| !page.account && page.scene_id == scene_id)
            .map(|page| page.id)
    }

    pub(crate) fn scenes(&self) -> Vec<ApplicationSceneItem> {
        let mut seen = HashSet::new();
        self.pages
            .iter()
            .filter(|page| !page.account && seen.insert(page.scene_id))
            .map(|page| ApplicationSceneItem {
                id: page.scene_id.to_owned(),
                label: page.scene_label.to_owned(),
            })
            .collect()
    }

    pub(crate) fn menus(&self, scene_id: Option<&str>) -> Vec<ApplicationMenuItem> {
        let mut menus = Vec::new();
        for page in self
            .pages
            .iter()
            .filter(|page| !page.account && Some(page.scene_id) == scene_id)
        {
            insert_page(&mut menus, page.menu_path, page);
        }
        menus
    }
}

fn insert_page(
    menus: &mut Vec<ApplicationMenuItem>,
    groups: &[ApplicationMenuGroup],
    page: &NavigationPage<'_>,
) {
    let Some((group, descendants)) = groups.split_first() else {
        menus.push(ApplicationMenuItem {
            id: page.id.to_owned(),
            label: page.label.to_owned(),
            icon: page.icon.map(str::to_owned),
            page_id: Some(page.id.to_owned()),
            enabled: true,
            children: Vec::new(),
        });
        return;
    };

    let group_index = menus
        .iter()
        .position(|menu| menu.id == group.id)
        .unwrap_or_else(|| {
            menus.push(ApplicationMenuItem {
                id: group.id.clone(),
                label: group.label.clone(),
                icon: group.icon.clone(),
                page_id: None,
                enabled: true,
                children: Vec::new(),
            });
            menus.len() - 1
        });
    insert_page(&mut menus[group_index].children, descendants, page);
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use dioxus::prelude::*;

    use super::*;

    fn page(
        id: &'static str,
        scene: &'static str,
        menu_path: Vec<ApplicationMenuGroup>,
    ) -> ApplicationPage {
        ApplicationPage {
            id,
            label: id,
            icon: None,
            scene: crate::ApplicationScene {
                id: scene,
                label: scene,
            },
            menu_path,
            required_permission: None,
            render: || rsx! { p { "页面" } },
        }
    }

    fn group(id: &str, label: &str) -> ApplicationMenuGroup {
        ApplicationMenuGroup {
            id: id.to_owned(),
            label: label.to_owned(),
            icon: None,
        }
    }

    fn account(page_id: &str) -> ApplicationAccountItem {
        ApplicationAccountItem {
            id: format!("open-{page_id}"),
            label: page_id.to_owned(),
            icon: None,
            page_id: Some(page_id.to_owned()),
            required_permission: None,
            destructive: false,
        }
    }

    fn runtime_page(
        id: &str,
        scene: &str,
        menu_path: Vec<ApplicationMenuGroup>,
    ) -> ApplicationRuntimePage {
        ApplicationRuntimePage {
            id: id.to_owned(),
            label: id.to_owned(),
            icon: None,
            scene_id: scene.to_owned(),
            scene_label: scene.to_owned(),
            menu_path,
            required_permission: None,
            definition: "{}".to_owned(),
        }
    }

    #[test]
    fn scene_is_root_and_plugins_merge_nested_group_paths() -> Result<()> {
        let pages = [
            page("home", "workspace", Vec::new()),
            page(
                "users",
                "system",
                vec![group("system-management", "系统管理")],
            ),
            page(
                "files",
                "system",
                vec![
                    group("infrastructure", "基础设施"),
                    group("file-management", "文件管理"),
                ],
            ),
        ];
        let runtime_pages = [runtime_page(
            "dictionary",
            "system",
            vec![group("system-management", "系统管理")],
        )];
        let navigation = PluginNavigation::new(&pages, &runtime_pages, &[])?;

        assert_eq!(navigation.scenes().len(), 2);
        let menus = navigation.menus(Some("system"));
        assert_eq!(menus.len(), 2);
        assert_eq!(menus[0].id, "system-management");
        assert_eq!(menus[0].page_id, None);
        assert_eq!(
            menus[0]
                .children
                .iter()
                .map(|menu| menu.id.as_str())
                .collect::<Vec<_>>(),
            ["users", "dictionary"]
        );
        assert_eq!(menus[1].id, "infrastructure");
        assert_eq!(menus[1].children[0].id, "file-management");
        assert_eq!(menus[1].children[0].children[0].id, "files");
        assert_eq!(navigation.first_page_in_scene("system"), Some("users"));
        assert!(navigation.menus(Some("removed")).is_empty());
        Ok(())
    }

    #[test]
    fn account_pages_are_fullscreen_destinations_without_sidebar_entries() -> Result<()> {
        let pages = [
            page(
                "settings",
                "account",
                vec![group("system-management", "不会进入侧栏")],
            ),
            page("home", "workspace", Vec::new()),
        ];
        let navigation = PluginNavigation::new(&pages, &[], &[account("settings")])?;
        assert_eq!(navigation.scenes().len(), 1);
        assert_eq!(navigation.scenes()[0].id, "workspace");
        assert!(navigation.menus(Some("account")).is_empty());
        assert_eq!(navigation.workspace_page(Some("settings")), Some("home"));
        assert_eq!(navigation.account_page(Some("settings")), Some("settings"));
        assert_eq!(navigation.account_page(Some("home")), None);
        Ok(())
    }

    #[test]
    fn removed_pages_fall_back_and_stale_account_targets_do_not_open() -> Result<()> {
        let pages = [page("home", "workspace", Vec::new())];
        let navigation = PluginNavigation::new(&pages, &[], &[account("uninstalled")])?;
        assert_eq!(navigation.workspace_page(Some("uninstalled")), Some("home"));
        assert_eq!(navigation.account_page(Some("uninstalled")), None);
        assert_eq!(navigation.scene_for_page(Some("home")), Some("workspace"));
        assert_eq!(
            PluginNavigation::new(&[], &[], &[])?.workspace_page(None),
            None
        );
        Ok(())
    }

    #[test]
    fn runtime_account_contributions_follow_the_same_navigation_contract() -> Result<()> {
        let runtime_pages = [runtime_page("profile-addon", "community", Vec::new())];
        let navigation = PluginNavigation::new(&[], &runtime_pages, &[account("profile-addon")])?;
        assert!(navigation.scenes().is_empty());
        assert_eq!(navigation.workspace_page(None), None);
        assert_eq!(
            navigation.account_page(Some("profile-addon")),
            Some("profile-addon")
        );
        Ok(())
    }

    #[test]
    fn rejects_runtime_group_conflicts_and_cycles() -> Result<()> {
        let conflict = [
            runtime_page(
                "users",
                "system",
                vec![group("system-management", "系统管理")],
            ),
            runtime_page(
                "roles",
                "system",
                vec![group("system-management", "另一个标题")],
            ),
        ];
        let conflict_error = PluginNavigation::new(&[], &conflict, &[])
            .err()
            .ok_or_else(|| anyhow::anyhow!("分组冲突必须被拒绝"))?;
        assert!(conflict_error.to_string().contains("展示信息不一致"));

        let cycle = [runtime_page(
            "dictionary",
            "system",
            vec![
                group("system-management", "系统管理"),
                group("system-management", "系统管理"),
            ],
        )];
        let cycle_error = PluginNavigation::new(&[], &cycle, &[])
            .err()
            .ok_or_else(|| anyhow::anyhow!("循环路径必须被拒绝"))?;
        assert!(cycle_error.to_string().contains("形成循环"));
        Ok(())
    }
}
