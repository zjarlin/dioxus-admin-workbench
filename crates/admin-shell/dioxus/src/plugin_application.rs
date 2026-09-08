use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    ApplicationAccountItem, ApplicationMenuItem, ApplicationPage, ApplicationRuntimePage,
    ApplicationSceneItem, ApplicationShell, ApplicationUser,
};

/// 将插件页面编排为可容纳多场景、多页面的应用壳。
#[component]
pub fn PluginApplication(
    application_label: String,
    pages: Vec<ApplicationPage>,
    user: ApplicationUser,
    #[props(default)] runtime_pages: Vec<ApplicationRuntimePage>,
    #[props(default)] render_runtime_page: Option<Callback<ApplicationRuntimePage, Element>>,
    #[props(default)] account_items: Vec<ApplicationAccountItem>,
    #[props(default)] on_account_action: Option<Callback<String>>,
) -> Element {
    let initial_page_id = pages
        .first()
        .map(|page| page.id.to_owned())
        .or_else(|| runtime_pages.first().map(|page| page.id.clone()));
    let mut active_page_id = use_signal(move || initial_page_id);
    let active_page_id_value = active_page_id();
    let selected_page = active_page_id_value
        .as_deref()
        .and_then(|id| pages.iter().find(|page| page.id == id));
    let selected_runtime_page = active_page_id_value
        .as_deref()
        .and_then(|id| runtime_pages.iter().find(|page| page.id == id));
    let active_page = selected_page.or_else(|| {
        selected_runtime_page
            .is_none()
            .then(|| pages.first())
            .flatten()
    });
    let active_runtime_page = selected_runtime_page.or_else(|| {
        active_page
            .is_none()
            .then(|| runtime_pages.first())
            .flatten()
    });
    let active_scene_id = active_page
        .map(|page| page.scene.id.to_owned())
        .or_else(|| active_runtime_page.map(|page| page.scene_id.clone()));
    let page_label = active_page
        .map(|page| page.label.to_owned())
        .or_else(|| active_runtime_page.map(|page| page.label.clone()))
        .unwrap_or_else(|| "暂无页面".to_owned());
    let scenes = application_scenes(&pages, &runtime_pages);
    let menus = application_menus(&pages, &runtime_pages);
    let content = active_page.map(|page| (page.render)()).or_else(|| {
        active_runtime_page
            .and_then(|page| render_runtime_page.map(|renderer| renderer.call(page.clone())))
    });
    let select_scene_pages = pages.clone();
    let select_scene_runtime_pages = runtime_pages.clone();
    let account_enabled = !account_items.is_empty();
    let account_action_items = account_items.clone();
    let account_action = Callback::new(move |action_id: String| {
        if let Some(page_id) = account_action_items
            .iter()
            .find(|item| item.id == action_id)
            .and_then(|item| item.page_id.as_deref())
        {
            active_page_id.set(Some(page_id.to_owned()));
            return;
        }
        if let Some(callback) = on_account_action {
            callback.call(action_id);
        }
    });

    rsx! {
        ApplicationShell {
            application_label,
            page_label,
            scenes,
            active_scene_id,
            menus,
            active_page_id: active_page_id(),
            user,
            account_enabled,
            on_select_scene: move |scene_id: String| {
                let next_page_id = select_scene_pages
                    .iter()
                    .find(|page| page.scene.id == scene_id)
                    .map(|page| page.id.to_owned())
                    .or_else(|| {
                        select_scene_runtime_pages
                            .iter()
                            .find(|page| page.scene_id == scene_id)
                            .map(|page| page.id.clone())
                    });
                active_page_id.set(next_page_id);
            },
            on_select_page: move |page_id: String| active_page_id.set(Some(page_id)),
            on_account_action: account_action,
            account_items,
            {content}
        }
    }
}

fn application_scenes(
    pages: &[ApplicationPage],
    runtime_pages: &[ApplicationRuntimePage],
) -> Vec<ApplicationSceneItem> {
    let mut seen = HashSet::new();
    let mut scenes = Vec::new();
    for page in pages {
        if seen.insert(page.scene.id) {
            scenes.push(ApplicationSceneItem {
                id: page.scene.id.to_owned(),
                label: page.scene.label.to_owned(),
            });
        }
    }
    for page in runtime_pages {
        if seen.insert(page.scene_id.as_str()) {
            scenes.push(ApplicationSceneItem {
                id: page.scene_id.clone(),
                label: page.scene_label.clone(),
            });
        }
    }
    scenes
}

fn application_menus(
    pages: &[ApplicationPage],
    runtime_pages: &[ApplicationRuntimePage],
) -> Vec<ApplicationMenuItem> {
    let mut group_indexes = std::collections::HashMap::<String, usize>::new();
    let mut groups = Vec::<ApplicationMenuItem>::new();
    for page in pages {
        let index = scene_group(
            &mut groups,
            &mut group_indexes,
            page.scene.id,
            page.scene.label,
        );
        groups[index].children.push(ApplicationMenuItem {
            id: page.id.to_owned(),
            label: page.label.to_owned(),
            icon: page.icon.map(str::to_owned),
            page_id: Some(page.id.to_owned()),
            enabled: true,
            children: Vec::new(),
        });
    }
    for page in runtime_pages {
        let index = scene_group(
            &mut groups,
            &mut group_indexes,
            &page.scene_id,
            &page.scene_label,
        );
        groups[index].children.push(ApplicationMenuItem {
            id: page.id.clone(),
            label: page.label.clone(),
            icon: page.icon.clone(),
            page_id: Some(page.id.clone()),
            enabled: true,
            children: Vec::new(),
        });
    }
    groups
}

fn scene_group(
    groups: &mut Vec<ApplicationMenuItem>,
    indexes: &mut std::collections::HashMap<String, usize>,
    id: &str,
    label: &str,
) -> usize {
    if let Some(index) = indexes.get(id) {
        return *index;
    }
    let index = groups.len();
    groups.push(ApplicationMenuItem {
        id: format!("scene-{id}"),
        label: label.to_owned(),
        icon: None,
        page_id: None,
        enabled: true,
        children: Vec::new(),
    });
    indexes.insert(id.to_owned(), index);
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_page() -> Element {
        rsx! { p { "页面" } }
    }

    fn page(id: &'static str, scene_id: &'static str) -> ApplicationPage {
        ApplicationPage {
            id,
            label: id,
            icon: None,
            scene: crate::ApplicationScene {
                id: scene_id,
                label: scene_id,
            },
            required_permission: None,
            render: render_page,
        }
    }

    #[test]
    fn derives_scene_tabs_and_active_scene_menus() {
        let pages = vec![
            page("home", "workspace"),
            page("orders", "workspace"),
            page("settings", "system"),
        ];

        let runtime_pages = vec![ApplicationRuntimePage {
            id: "runtime".to_owned(),
            label: "运行时".to_owned(),
            icon: None,
            scene_id: "plugins".to_owned(),
            scene_label: "插件".to_owned(),
            required_permission: None,
            definition: "{}".to_owned(),
        }];
        let scenes = application_scenes(&pages, &runtime_pages);
        let menus = application_menus(&pages, &runtime_pages);

        assert_eq!(scenes.len(), 3);
        assert_eq!(menus.len(), 3);
        assert_eq!(menus[0].children.len(), 2);
        assert_eq!(menus[0].children[1].page_id.as_deref(), Some("orders"));
    }
}
