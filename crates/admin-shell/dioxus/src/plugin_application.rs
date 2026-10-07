use dioxus::prelude::*;
use std::collections::BTreeMap;

use crate::{
    ApplicationAccountItem, ApplicationPage, ApplicationRuntimePage, ApplicationShell,
    ApplicationTopbarItem, ApplicationUser,
    page_cache::{PageScope, PageSource},
    page_deck::PageDeck,
    plugin_navigation::PluginNavigation,
    url_state::{UrlUpdate, use_url_navigation},
};

/// 将插件页面编排为场景菜单树与独立账户页面。
#[component]
pub fn PluginApplication(
    application_label: String,
    pages: Vec<ApplicationPage>,
    user: ApplicationUser,
    #[props(default)] runtime_pages: Vec<ApplicationRuntimePage>,
    #[props(default)] runtime_page_versions: BTreeMap<String, String>,
    #[props(default)] prepared_pages: Vec<String>,
    #[props(default)] workspace_id: String,
    #[props(default)] workspace_context: String,
    #[props(default)] render_runtime_page: Option<Callback<ApplicationRuntimePage, Element>>,
    #[props(default)] account_items: Vec<ApplicationAccountItem>,
    #[props(default)] on_account_action: Option<Callback<String>>,
    #[props(default)] topbar_items: Vec<ApplicationTopbarItem>,
    #[props(default = 6)] workspace_cache_capacity: usize,
    #[props(default = 2)] account_cache_capacity: usize,
) -> Element {
    let url = use_url_navigation();
    let scope = PageScope {
        id: workspace_id.clone(),
        version: workspace_context,
    };
    let navigation = match PluginNavigation::new(&pages, &runtime_pages, &account_items) {
        Ok(navigation) => navigation,
        Err(error) => {
            return rsx! {
                az_ui_components::UiStylesheets {}
                section { class: "application-shell__state", role: "alert",
                    "插件导航无效：{error}"
                }
            };
        }
    };
    let location = url.location.read().clone();
    let selected_page = location.as_ref().and_then(|value| value.page.clone());
    let selected_account_page = location.as_ref().and_then(|value| value.account.clone());
    let resolved_page = navigation.workspace_page(selected_page.as_deref());
    let invalid_page = selected_page.is_some() && resolved_page != selected_page.as_deref();
    let workspace_page = if invalid_page { None } else { resolved_page };
    let fullscreen_page = navigation.account_page(selected_account_page.as_deref());
    let default_page = navigation.workspace_page(None).map(str::to_owned);
    let scope_key = format!("{}:{}", scope.id, scope.version);
    let initial_page = default_page.clone();
    use_effect(move || {
        let location = url.location.read();
        if let Some(value) = location.as_ref() {
            url.scope(&scope_key, initial_page.as_deref());
            if value.page.is_none() && initial_page.is_some() {
                url.navigate(
                    initial_page.as_deref(),
                    value.account.as_deref(),
                    UrlUpdate::Replace,
                );
            }
        }
    });
    if location.is_none() {
        return rsx! { az_ui_components::admin::RequestState {} };
    }
    let unavailable =
        invalid_page || (selected_account_page.is_some() && fullscreen_page.is_none());
    let active_scene_id = navigation
        .scene_for_page(workspace_page.or(default_page.as_deref()))
        .map(str::to_owned);
    let scenes = navigation.scenes();
    let menus = navigation.menus(active_scene_id.as_deref());
    let scene_destinations = scenes
        .iter()
        .filter_map(|scene| {
            navigation
                .first_page_in_scene(&scene.id)
                .map(|page_id| (scene.id.clone(), page_id.to_owned()))
        })
        .collect::<Vec<_>>();
    let workspace_label = page_label(workspace_page, &pages, &runtime_pages);
    let sources = pages
        .iter()
        .cloned()
        .map(PageSource::Native)
        .chain(runtime_pages.iter().cloned().map(|page| {
            let version = runtime_page_versions
                .get(&page.id)
                .cloned()
                .unwrap_or_default();
            PageSource::Runtime(page, version)
        }))
        .collect::<Vec<_>>();
    let account_enabled = !account_items.is_empty();
    let account_action_items = account_items.clone();
    let valid_account_pages = account_items
        .iter()
        .filter_map(|item| {
            navigation
                .account_page(item.page_id.as_deref())
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    let account_workspace_page = workspace_page.map(str::to_owned);
    let account_action = Callback::new(move |action_id: String| {
        if let Some(item) = account_action_items
            .iter()
            .find(|item| item.id == action_id)
        {
            if let Some(page_id) = &item.page_id {
                if valid_account_pages.contains(page_id) {
                    url.navigate(
                        account_workspace_page.as_deref(),
                        Some(page_id),
                        UrlUpdate::Push,
                    );
                }
                return;
            }
            if let Some(callback) = on_account_action {
                callback.call(action_id);
            }
        }
    });
    let background_page = workspace_page.map(str::to_owned);

    rsx! {
        // 账户页打开时保留后台组件实例，返回后恢复页面内部状态。
        div { hidden: fullscreen_page.is_some(),
            ApplicationShell {
                application_label: application_label.clone(),
                page_label: workspace_label,
                scenes,
                active_scene_id,
                menus,
                active_page_id: workspace_page.map(str::to_owned),
                user,
                account_enabled,
                on_select_scene: move |scene_id: String| {
                    if let Some((_, page_id)) = scene_destinations.iter().find(|(id, _)| id == &scene_id) {
                        url.navigate(Some(page_id), None, UrlUpdate::Push);
                    }
                },
                on_select_page: move |page_id: String| url.navigate(Some(&page_id), None, UrlUpdate::Push),
                on_account_action: account_action,
                account_items,
                topbar_items,
                if unavailable {
                    section { class: "application-shell__state", role: "alert",
                        p { "页面不存在或没有访问权限" }
                        az_ui_components::button::Button {
                            onclick: move |_| url.navigate(default_page.as_deref(), None, UrlUpdate::Push),
                            "返回可用页面"
                        }
                    }
                }
                PageDeck {
                    pages: sources.clone(), selected: if unavailable { None } else { workspace_page.map(str::to_owned) },
                    active: fullscreen_page.is_none() && !unavailable, capacity: workspace_cache_capacity,
                    scope: scope.clone(),
                    renderer: render_runtime_page,
                    prepared_pages,
                }
            }
        }
        PageDeck {
            pages: sources, selected: fullscreen_page.map(str::to_owned),
            active: fullscreen_page.is_some(), capacity: account_cache_capacity,
            scope,
            renderer: render_runtime_page, fullscreen: application_label,
            on_back: move |_| url.navigate(background_page.as_deref(), None, UrlUpdate::Push),
        }
    }
}

fn page_label(
    selected: Option<&str>,
    pages: &[ApplicationPage],
    runtime_pages: &[ApplicationRuntimePage],
) -> String {
    pages
        .iter()
        .find(|page| Some(page.id) == selected)
        .map(|page| page.label.to_owned())
        .or_else(|| {
            runtime_pages
                .iter()
                .find(|page| Some(page.id.as_str()) == selected)
                .map(|page| page.label.clone())
        })
        .unwrap_or_else(|| "暂无页面".to_owned())
}
