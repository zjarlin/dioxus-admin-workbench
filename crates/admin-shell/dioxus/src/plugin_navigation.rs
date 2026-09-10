use std::collections::HashSet;

use crate::{
    ApplicationAccountItem, ApplicationMenuItem, ApplicationPage, ApplicationRuntimePage,
    ApplicationSceneItem,
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
    account: bool,
}

impl<'a> PluginNavigation<'a> {
    pub(crate) fn new(
        pages: &'a [ApplicationPage],
        runtime_pages: &'a [ApplicationRuntimePage],
        account_items: &[ApplicationAccountItem],
    ) -> Self {
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
                account: account_pages.contains(page.id),
            })
            .chain(runtime_pages.iter().map(|page| NavigationPage {
                id: &page.id,
                label: &page.label,
                icon: page.icon.as_deref(),
                scene_id: &page.scene_id,
                scene_label: &page.scene_label,
                account: account_pages.contains(page.id.as_str()),
            }))
            .collect();
        Self { pages }
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
        self.pages
            .iter()
            .filter(|page| !page.account && Some(page.scene_id) == scene_id)
            .map(|page| ApplicationMenuItem {
                id: page.id.to_owned(),
                label: page.label.to_owned(),
                icon: page.icon.map(str::to_owned),
                page_id: Some(page.id.to_owned()),
                enabled: true,
                children: Vec::new(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    fn page(id: &'static str, scene: &'static str) -> ApplicationPage {
        ApplicationPage {
            id,
            label: id,
            icon: None,
            scene: crate::ApplicationScene {
                id: scene,
                label: scene,
            },
            required_permission: None,
            render: || rsx! { p { "页面" } },
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

    fn runtime_page(id: &str, scene: &str) -> ApplicationRuntimePage {
        ApplicationRuntimePage {
            id: id.to_owned(),
            label: id.to_owned(),
            icon: None,
            scene_id: scene.to_owned(),
            scene_label: scene.to_owned(),
            required_permission: None,
            definition: "{}".to_owned(),
        }
    }

    #[test]
    fn scene_is_the_tree_root_instead_of_a_sidebar_group() {
        let pages = [page("home", "workspace"), page("users", "system")];
        let runtime_pages = [runtime_page("hello", "workspace")];
        let navigation = PluginNavigation::new(&pages, &runtime_pages, &[]);
        assert_eq!(navigation.scenes().len(), 2);
        let menus = navigation.menus(Some("workspace"));
        assert_eq!(
            menus
                .iter()
                .map(|menu| menu.id.as_str())
                .collect::<Vec<_>>(),
            ["home", "hello"]
        );
        assert!(menus.iter().all(|menu| menu.children.is_empty()));
        assert_eq!(navigation.menus(Some("system"))[0].id, "users");
        assert!(navigation.menus(Some("removed")).is_empty());
    }

    #[test]
    fn account_pages_are_fullscreen_destinations_without_sidebar_entries() {
        let pages = [page("settings", "account"), page("home", "workspace")];
        let navigation = PluginNavigation::new(&pages, &[], &[account("settings")]);
        assert_eq!(navigation.scenes().len(), 1);
        assert_eq!(navigation.scenes()[0].id, "workspace");
        assert!(navigation.menus(Some("account")).is_empty());
        assert_eq!(navigation.workspace_page(Some("settings")), Some("home"));
        assert_eq!(navigation.account_page(Some("settings")), Some("settings"));
        assert_eq!(navigation.account_page(Some("home")), None);
    }

    #[test]
    fn removed_pages_fall_back_and_stale_account_targets_do_not_open() {
        let pages = [page("home", "workspace")];
        let navigation = PluginNavigation::new(&pages, &[], &[account("uninstalled")]);
        assert_eq!(navigation.workspace_page(Some("uninstalled")), Some("home"));
        assert_eq!(navigation.account_page(Some("uninstalled")), None);
        assert_eq!(navigation.scene_for_page(Some("home")), Some("workspace"));
        assert_eq!(
            PluginNavigation::new(&[], &[], &[]).workspace_page(None),
            None
        );
    }

    #[test]
    fn runtime_account_contributions_follow_the_same_navigation_contract() {
        let runtime_pages = [runtime_page("profile-addon", "community")];
        let navigation = PluginNavigation::new(&[], &runtime_pages, &[account("profile-addon")]);
        assert!(navigation.scenes().is_empty());
        assert_eq!(navigation.workspace_page(None), None);
        assert_eq!(
            navigation.account_page(Some("profile-addon")),
            Some("profile-addon")
        );
    }
}
