use az_ui_components::{
    button::{Button, ButtonVariant},
    navigation_icon::{NavigationIcon, resolved_navigation_icon},
};
use dioxus::prelude::*;

use crate::ApplicationMenuItem;

#[component]
pub(crate) fn ApplicationNavigation(
    menus: Vec<ApplicationMenuItem>,
    active_page_id: Option<String>,
    on_select_page: Callback<String>,
) -> Element {
    rsx! {
        nav { class: "application-shell__navigation", aria_label: "应用页面",
            for menu in menus.into_iter().filter(|menu| menu.enabled) {
                ApplicationNavigationItem {
                    key: "{menu.id}",
                    menu,
                    depth: 0,
                    active_page_id: active_page_id.clone(),
                    on_select_page,
                }
            }
        }
    }
}

#[component]
fn ApplicationNavigationItem(
    menu: ApplicationMenuItem,
    depth: usize,
    active_page_id: Option<String>,
    on_select_page: Callback<String>,
) -> Element {
    let page_id = menu.page_id.clone();
    let icon = resolved_navigation_icon(menu.icon.as_deref(), &menu.label).to_owned();
    let children = menu
        .children
        .into_iter()
        .filter(|child| child.enabled)
        .collect::<Vec<_>>();
    let is_group = page_id.is_none();
    rsx! {
        section {
            class: if is_group {
                "application-shell__navigation-group"
            } else {
                "application-shell__navigation-item"
            },
            if let Some(page_id) = page_id {
                Button {
                    class: "application-shell__navigation-button",
                    r#type: "button",
                    variant: if active_page_id.as_deref() == Some(page_id.as_str()) {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    },
                    title: menu.label.clone(),
                    aria_label: menu.label.clone(),
                    onclick: move |_| on_select_page.call(page_id.clone()),
                    span { class: "application-shell__navigation-icon", aria_hidden: "true",
                        NavigationIcon { name: icon, class: "size-4".to_owned() }
                    }
                    span { class: "application-shell__navigation-label", "{menu.label}" }
                }
            } else {
                div { class: "application-shell__navigation-heading",
                    span { class: "application-shell__navigation-icon", aria_hidden: "true",
                        NavigationIcon { name: icon, class: "size-4".to_owned() }
                    }
                    span { class: "application-shell__navigation-label", "{menu.label}" }
                }
            }
            if !children.is_empty() {
                div {
                    class: "application-shell__navigation-children",
                    "data-depth": depth.to_string(),
                    for child in children {
                        ApplicationNavigationItem {
                            key: "{child.id}",
                            menu: child,
                            depth: depth + 1,
                            active_page_id: active_page_id.clone(),
                            on_select_page,
                        }
                    }
                }
            }
        }
    }
}
