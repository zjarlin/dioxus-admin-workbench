use az_admin_shell_core::AdminCommand;
use az_ui_components::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
    input::Input,
};
use dioxus::prelude::*;

use crate::AdminProviderHandle;

#[component]
pub(crate) fn ApplicationDialog(
    admin: AdminProviderHandle,
    current_title: String,
    mut open: Signal<bool>,
    mut generation: Signal<u64>,
    on_status: Callback<String>,
) -> Element {
    let mut title = use_signal(move || current_title);
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        Dialog {
            class: "admin-shell__application-dialog",
            open: open(),
            on_open_change: move |value| open.set(value),
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let next_title = title().trim().to_owned();
                    if next_title.is_empty() {
                        error.set(Some("应用标题不能为空".to_owned()));
                        return;
                    }
                    let provider = admin.provider().clone();
                    pending.set(true);
                    spawn(async move {
                        let command = AdminCommand::SetApplicationTitle {
                            title: next_title.clone(),
                        };
                        match provider.execute(command).await {
                            Ok(_) => {
                                generation.with_mut(|value| *value = value.saturating_add(1));
                                on_status.call(format!("已更新应用标题：{next_title}"));
                                open.set(false);
                            }
                            Err(cause) => error.set(Some(cause.to_string())),
                        }
                        pending.set(false);
                    });
                },
                header {
                    DialogTitle { "编辑应用" }
                    DialogDescription { "应用标题显示在工作台左上角。" }
                }
                label { r#for: "application-title", "应用标题" }
                Input {
                    id: "application-title",
                    name: "title",
                    aria_label: "应用标题",
                    value: "{title}",
                    oninput: move |event: FormEvent| title.set(event.value()),
                }
                if let Some(message) = error() {
                    p { role: "alert", "{message}" }
                }
                footer {
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| open.set(false),
                        "取消"
                    }
                    Button { r#type: "submit", disabled: pending(), "保存" }
                }
            }
        }
    }
}
