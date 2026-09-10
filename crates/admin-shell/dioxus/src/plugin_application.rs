use dioxus::prelude::*;

use crate::{
    ApplicationAccountItem, ApplicationFullscreenPage, ApplicationPage, ApplicationRuntimePage,
    ApplicationShell, ApplicationUser, plugin_navigation::PluginNavigation,
};

/// 将插件页面编排为场景菜单树与独立账户页面。
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
    let mut active_page_id = use_signal(|| None::<String>);
    let mut account_page_id = use_signal(|| None::<String>);
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
    let selected_page = active_page_id();
    let selected_account_page = account_page_id();
    let workspace_page = navigation.workspace_page(selected_page.as_deref());
    let fullscreen_page = navigation.account_page(selected_account_page.as_deref());
    let active_scene_id = navigation.scene_for_page(workspace_page).map(str::to_owned);
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
    let fullscreen_label = page_label(fullscreen_page, &pages, &runtime_pages);
    let content = page_content(workspace_page, &pages, &runtime_pages, render_runtime_page);
    let fullscreen_content =
        page_content(fullscreen_page, &pages, &runtime_pages, render_runtime_page);
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
    let account_action = Callback::new(move |action_id: String| {
        if let Some(item) = account_action_items
            .iter()
            .find(|item| item.id == action_id)
        {
            if let Some(page_id) = &item.page_id {
                if valid_account_pages.contains(page_id) {
                    account_page_id.set(Some(page_id.clone()));
                }
                return;
            }
            if let Some(callback) = on_account_action {
                callback.call(action_id);
            }
        }
    });

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
                        active_page_id.set(Some(page_id.clone()));
                    }
                },
                on_select_page: move |page_id: String| active_page_id.set(Some(page_id)),
                on_account_action: account_action,
                account_items,
                {content}
            }
        }
        if let Some(page_id) = fullscreen_page {
            ApplicationFullscreenPage {
                key: "{page_id}",
                application_label,
                page_label: fullscreen_label,
                on_back: move |_| account_page_id.set(None),
                {fullscreen_content}
            }
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

fn page_content(
    selected: Option<&str>,
    pages: &[ApplicationPage],
    runtime_pages: &[ApplicationRuntimePage],
    render_runtime_page: Option<Callback<ApplicationRuntimePage, Element>>,
) -> Element {
    if let Some(page) = pages.iter().find(|page| Some(page.id) == selected) {
        return rsx! { ApplicationPluginPage { key: "{page.id}", render: page.render } };
    }
    if let Some(page) = runtime_pages
        .iter()
        .find(|page| Some(page.id.as_str()) == selected)
        && let Some(renderer) = render_runtime_page
    {
        return renderer.call(page.clone());
    }
    rsx! {}
}

#[derive(Clone, Props)]
struct ApplicationPluginPageProps {
    render: fn() -> Element,
}

impl PartialEq for ApplicationPluginPageProps {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.render, other.render)
    }
}

#[allow(non_snake_case)]
fn ApplicationPluginPage(props: ApplicationPluginPageProps) -> Element {
    (props.render)()
}
