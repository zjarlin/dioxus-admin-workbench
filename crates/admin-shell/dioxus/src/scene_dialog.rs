use az_admin_shell_core::{AdminCommand, DefinitionId, SceneDefinition, identifier_from_title};
use az_ui_components::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
    input::Input,
};
use dioxus::prelude::*;

use crate::AdminProviderHandle;

#[component]
pub(crate) fn SceneDialog(
    admin: AdminProviderHandle,
    mut open: Signal<bool>,
    mut generation: Signal<u64>,
    on_status: Callback<String>,
) -> Element {
    let mut title = use_signal(String::new);
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        Dialog {
            open: open(),
            on_open_change: move |value| open.set(value),
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let scene_title = title().trim().to_owned();
                    if scene_title.is_empty() {
                        error.set(Some("场景标题不能为空".to_owned()));
                        return;
                    }
                    let scene_name = identifier_from_title(&scene_title);
                    if scene_name.is_empty() {
                        error.set(Some("场景标题无法生成有效标识，请包含中文、字母或数字".to_owned()));
                        return;
                    }
                    let provider = admin.provider().clone();
                    pending.set(true);
                    spawn(async move {
                        let command = AdminCommand::AddScene {
                            scene: SceneDefinition {
                                id: DefinitionId::new(),
                                name: scene_name,
                                title: scene_title.clone(),
                                menus: Vec::new(),
                            },
                        };
                        match provider.execute(command).await {
                            Ok(_) => {
                                generation.with_mut(|value| *value = value.saturating_add(1));
                                on_status.call(format!("已创建场景：{scene_title}"));
                                open.set(false);
                            }
                            Err(cause) => error.set(Some(cause.to_string())),
                        }
                        pending.set(false);
                    });
                },
                header {
                    DialogTitle { "新建场景" }
                    DialogDescription { "场景是顶层业务上下文，内部组织菜单和页面。" }
                }
                label { r#for: "scene-title", "场景标题" }
                Input {
                    id: "scene-title",
                    name: "title",
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
                    Button { r#type: "submit", disabled: pending(), "创建场景" }
                }
            }
        }
    }
}
