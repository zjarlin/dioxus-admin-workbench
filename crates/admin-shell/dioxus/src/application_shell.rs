use az_ui_components::{
    UiStylesheets,
    badge::{Badge, BadgeVariant},
    button::{Button, ButtonSize, ButtonVariant},
    dialog::{Dialog, DialogTitle},
};
use dioxus::prelude::*;
use icons::{PanelLeft, Plus, Settings, X};

use crate::{
    ApplicationAccountAction, ApplicationMenuItem, ApplicationSceneItem, ApplicationUser,
    application_account::ApplicationAccountMenu, application_navigation::ApplicationNavigation,
};

#[component]
pub fn ApplicationShell(
    application_label: String,
    page_label: String,
    scenes: Vec<ApplicationSceneItem>,
    active_scene_id: Option<String>,
    menus: Vec<ApplicationMenuItem>,
    active_page_id: Option<String>,
    user: ApplicationUser,
    on_select_scene: Callback<String>,
    on_select_page: Callback<String>,
    on_account_action: Callback<ApplicationAccountAction>,
    #[props(default)] status: Option<String>,
    #[props(default)] on_create_scene: Option<Callback<()>>,
    #[props(default)] on_create_menu: Option<Callback<()>>,
    #[props(default)] on_configure_page: Option<Callback<()>>,
    children: Element,
) -> Element {
    let mut sidebar_collapsed = use_signal(|| false);
    let mut mobile_navigation_open = use_signal(|| false);
    let mut account_menu_open = use_signal(|| false);
    let shell_select_page = Callback::new(move |page_id: String| {
        account_menu_open.set(false);
        mobile_navigation_open.set(false);
        on_select_page.call(page_id);
    });
    let shell_account_action = Callback::new(move |action| {
        account_menu_open.set(false);
        on_account_action.call(action);
    });
    rsx! {
        UiStylesheets {}
        section {
            class: "application-shell",
            "data-sidebar-collapsed": sidebar_collapsed().to_string(),
            aside { class: "application-shell__sidebar",
                header { class: "application-shell__sidebar-header",
                    Button {
                        r#type: "button",
                        size: ButtonSize::IconSm,
                        variant: ButtonVariant::Ghost,
                        title: if sidebar_collapsed() { "展开菜单" } else { "收起菜单" },
                        aria_label: if sidebar_collapsed() { "展开菜单" } else { "收起菜单" },
                        onclick: move |_| {
                            account_menu_open.set(false);
                            sidebar_collapsed.toggle();
                        },
                        PanelLeft { class: "size-5" }
                    }
                    strong { class: "application-shell__brand", "{application_label}" }
                }
                ApplicationNavigationPanel {
                    menus: menus.clone(),
                    active_page_id: active_page_id.clone(),
                    user: user.clone(),
                    account_menu_open,
                    on_select_page: shell_select_page,
                    on_account_action: shell_account_action,
                    on_create_menu,
                }
            }
            main { class: "application-shell__main",
                header { class: "application-shell__topbar",
                    div { class: "application-shell__page-heading",
                        Button {
                            class: "application-shell__mobile-trigger",
                            r#type: "button",
                            size: ButtonSize::IconSm,
                            variant: ButtonVariant::Ghost,
                            title: "打开菜单",
                            aria_label: "打开菜单",
                            onclick: move |_| mobile_navigation_open.set(true),
                            PanelLeft { class: "size-4" }
                        }
                        h1 { "{page_label}" }
                    }
                    nav { class: "application-shell__scenes", aria_label: "场景",
                        for scene in scenes {
                            Button {
                                key: "{scene.id}",
                                r#type: "button",
                                size: ButtonSize::Sm,
                                variant: if active_scene_id.as_deref() == Some(scene.id.as_str()) {
                                    ButtonVariant::Secondary
                                } else {
                                    ButtonVariant::Ghost
                                },
                                onclick: move |_| on_select_scene.call(scene.id.clone()),
                                "{scene.label}"
                            }
                        }
                        if let Some(create_scene) = on_create_scene {
                            Button {
                                r#type: "button",
                                size: ButtonSize::IconSm,
                                variant: ButtonVariant::Outline,
                                title: "新建场景",
                                aria_label: "新建场景",
                                onclick: move |_| create_scene.call(()),
                                Plus { class: "size-4" }
                            }
                        }
                    }
                    div { class: "application-shell__page-actions",
                        if let Some(message) = status {
                            Badge { variant: BadgeVariant::Outline, "{message}" }
                        }
                        if let Some(configure_page) = on_configure_page {
                            Button {
                                r#type: "button",
                                size: ButtonSize::IconSm,
                                variant: ButtonVariant::Ghost,
                                title: "设置当前页面",
                                aria_label: "设置当前页面",
                                onclick: move |_| configure_page.call(()),
                                Settings { class: "size-4" }
                            }
                        }
                    }
                }
                section { class: "application-shell__content", {children} }
            }
        }
        if mobile_navigation_open() {
            Dialog {
                class: "application-shell__mobile-dialog",
                open: true,
                on_open_change: move |open| mobile_navigation_open.set(open),
                header { class: "application-shell__mobile-header",
                    div { class: "application-shell__mobile-title",
                        PanelLeft { class: "size-5" }
                        DialogTitle { "{application_label}" }
                    }
                    Button {
                        r#type: "button",
                        size: ButtonSize::IconSm,
                        variant: ButtonVariant::Ghost,
                        title: "关闭菜单",
                        aria_label: "关闭菜单",
                        onclick: move |_| mobile_navigation_open.set(false),
                        X { class: "size-4" }
                    }
                }
                ApplicationNavigationPanel {
                    menus,
                    active_page_id,
                    user,
                    account_menu_open,
                    on_select_page: shell_select_page,
                    on_account_action: shell_account_action,
                    on_create_menu,
                }
            }
        }
    }
}

#[component]
fn ApplicationNavigationPanel(
    menus: Vec<ApplicationMenuItem>,
    active_page_id: Option<String>,
    user: ApplicationUser,
    account_menu_open: Signal<bool>,
    on_select_page: Callback<String>,
    on_account_action: Callback<ApplicationAccountAction>,
    on_create_menu: Option<Callback<()>>,
) -> Element {
    rsx! {
        div { class: "application-shell__navigation-panel",
            ApplicationNavigation { menus, active_page_id, on_select_page }
            footer { class: "application-shell__sidebar-footer",
                if let Some(create_menu) = on_create_menu {
                    Button {
                        class: "application-shell__create-menu",
                        r#type: "button",
                        variant: ButtonVariant::Outline,
                        title: "新建菜单",
                        aria_label: "新建菜单",
                        onclick: move |_| create_menu.call(()),
                        Plus { class: "size-4" }
                        span { class: "application-shell__sidebar-label", "新建菜单" }
                    }
                }
                ApplicationAccountMenu { user, open: account_menu_open, on_action: on_account_action }
            }
        }
    }
}
