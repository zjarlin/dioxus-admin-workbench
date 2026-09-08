use az_ui_components::button::{Button, ButtonVariant};
use az_ui_components::navigation_icon::NavigationIcon;
use dioxus::prelude::*;
use icons::ChevronDown;

use crate::{ApplicationAccountItem, ApplicationUser};

#[component]
pub(crate) fn ApplicationAccountMenu(
    user: ApplicationUser,
    mut open: Signal<bool>,
    items: Vec<ApplicationAccountItem>,
    on_action: Callback<String>,
) -> Element {
    let menu_label = format!("打开 {} 的账户菜单", user.label);
    rsx! {
        section {
            class: "application-shell__account",
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    open.set(false);
                }
            },
            Button {
                class: "application-shell__account-trigger",
                r#type: "button",
                variant: ButtonVariant::Ghost,
                title: menu_label.clone(),
                aria_label: menu_label,
                aria_expanded: open().to_string(),
                onclick: move |_| open.toggle(),
                span { class: "application-shell__avatar", aria_hidden: "true", "{user.initials}" }
                span { class: "application-shell__account-label", "{user.label}" }
                ChevronDown { class: "application-shell__account-chevron" }
            }
            if open() {
                div {
                    class: "application-shell__account-dismiss",
                    aria_hidden: "true",
                    onclick: move |_| open.set(false),
                }
                aside { class: "application-shell__account-menu", role: "menu",
                    header { class: "application-shell__account-summary",
                        span { class: "application-shell__avatar application-shell__avatar--large", aria_hidden: "true", "{user.initials}" }
                        div { class: "application-shell__account-identity",
                            strong { "{user.label}" }
                            span { "{user.handle}" }
                        }
                    }
                    div { class: "application-shell__account-actions",
                        for item in items {
                            AccountActionButton { item, on_action }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AccountActionButton(item: ApplicationAccountItem, on_action: Callback<String>) -> Element {
    let item_id = item.id.clone();
    rsx! {
        Button {
            class: if item.destructive {
                "application-shell__account-action application-shell__account-action--destructive"
            } else {
                "application-shell__account-action"
            },
            r#type: "button",
            variant: ButtonVariant::Ghost,
            role: "menuitem",
            onclick: move |_| on_action.call(item_id.clone()),
            NavigationIcon {
                name: item.icon.as_deref().unwrap_or("user").to_owned(),
                class: "size-4".to_owned(),
            }
            span { "{item.label}" }
        }
    }
}
