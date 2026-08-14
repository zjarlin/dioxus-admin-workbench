use az_ui_components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use icons::{ChevronDown, KeyRound, LogOut, Settings, UserRound};

use crate::{ApplicationAccountAction, ApplicationUser};

#[component]
pub(crate) fn ApplicationAccountMenu(
    user: ApplicationUser,
    mut open: Signal<bool>,
    on_action: Callback<ApplicationAccountAction>,
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
                        AccountActionButton {
                            action: ApplicationAccountAction::AgentSettings,
                            label: "Agent 设置",
                            on_action,
                        }
                        AccountActionButton {
                            action: ApplicationAccountAction::Profile,
                            label: "个人资料",
                            on_action,
                        }
                        AccountActionButton {
                            action: ApplicationAccountAction::ChangePassword,
                            label: "修改密码",
                            on_action,
                        }
                    }
                    div { class: "application-shell__account-signout",
                        AccountActionButton {
                            action: ApplicationAccountAction::SignOut,
                            label: "退出系统",
                            destructive: true,
                            on_action,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AccountActionButton(
    action: ApplicationAccountAction,
    label: &'static str,
    #[props(default)] destructive: bool,
    on_action: Callback<ApplicationAccountAction>,
) -> Element {
    rsx! {
        Button {
            class: if destructive {
                "application-shell__account-action application-shell__account-action--destructive"
            } else {
                "application-shell__account-action"
            },
            r#type: "button",
            variant: ButtonVariant::Ghost,
            role: "menuitem",
            onclick: move |_| on_action.call(action),
            match action {
                ApplicationAccountAction::AgentSettings => rsx! { Settings { class: "size-4" } },
                ApplicationAccountAction::Profile => rsx! { UserRound { class: "size-4" } },
                ApplicationAccountAction::ChangePassword => rsx! { KeyRound { class: "size-4" } },
                ApplicationAccountAction::SignOut => rsx! { LogOut { class: "size-4" } },
            }
            span { "{label}" }
        }
    }
}
