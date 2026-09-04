use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    ApplicationAccountAction, ApplicationMenuItem, ApplicationPage, ApplicationSceneItem,
    ApplicationShell, ApplicationUser,
};

/// 将插件页面编排为可容纳多场景、多页面的应用壳。
#[component]
pub fn PluginApplication(
    application_label: String,
    pages: Vec<ApplicationPage>,
    user: ApplicationUser,
    #[props(default)] on_account_action: Option<Callback<ApplicationAccountAction>>,
) -> Element {
    let initial_page_id = pages.first().map(|page| page.id.to_owned());
    let mut active_page_id = use_signal(move || initial_page_id);
    let active_page = active_page_id()
        .as_deref()
        .and_then(|id| pages.iter().find(|page| page.id == id))
        .or_else(|| pages.first());
    let active_scene_id = active_page.map(|page| page.scene.id.to_owned());
    let page_label =
        active_page.map_or_else(|| "暂无页面".to_owned(), |page| page.label.to_owned());
    let scenes = application_scenes(&pages);
    let menus = application_menus(&pages, active_scene_id.as_deref());
    let content = active_page.map(|page| (page.render)());
    let select_scene_pages = pages.clone();

    rsx! {
        ApplicationShell {
            application_label,
            page_label,
            scenes,
            active_scene_id,
            menus,
            active_page_id: active_page_id(),
            user,
            on_select_scene: move |scene_id: String| {
                let next_page_id = select_scene_pages
                    .iter()
                    .find(|page| page.scene.id == scene_id)
                    .map(|page| page.id.to_owned());
                active_page_id.set(next_page_id);
            },
            on_select_page: move |page_id: String| active_page_id.set(Some(page_id)),
            on_account_action: move |action| {
                if let Some(callback) = on_account_action {
                    callback.call(action);
                }
            },
            {content}
        }
    }
}

fn application_scenes(pages: &[ApplicationPage]) -> Vec<ApplicationSceneItem> {
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
    scenes
}

fn application_menus(
    pages: &[ApplicationPage],
    active_scene_id: Option<&str>,
) -> Vec<ApplicationMenuItem> {
    pages
        .iter()
        .filter(|page| active_scene_id == Some(page.scene.id))
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

        let scenes = application_scenes(&pages);
        let menus = application_menus(&pages, Some("workspace"));

        assert_eq!(scenes.len(), 2);
        assert_eq!(menus.len(), 2);
        assert_eq!(menus[1].page_id.as_deref(), Some("orders"));
    }
}
