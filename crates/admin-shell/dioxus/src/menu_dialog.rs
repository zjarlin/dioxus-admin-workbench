use std::sync::Arc;

use az_admin_shell_core::{
    AdminCommand, DefinitionId, ExtensionType, MenuDefinition, PageDefinition,
    PageExtensionCompilerIndex, PageRendererDefinition, ResourceCatalog,
};
use az_ui_components::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
    input::Input,
    navigation_icon::{DEFAULT_NAVIGATION_ICON, NavigationIconPicker},
};
use dioxus::prelude::*;
use serde_json::Value;

use crate::{AdminProviderHandle, PageExtensionEditorContext, PageExtensionRendererIndex};

#[component]
pub(crate) fn MenuDialog(
    admin: AdminProviderHandle,
    scene_id: DefinitionId,
    resources: ResourceCatalog,
    compiler_extensions: Arc<PageExtensionCompilerIndex>,
    renderer_extensions: Arc<PageExtensionRendererIndex>,
    mut open: Signal<bool>,
    mut generation: Signal<u64>,
    on_status: Callback<String>,
) -> Element {
    let mut name = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut icon = use_signal(|| DEFAULT_NAVIGATION_ICON.to_owned());
    let mut renderer = use_signal(|| "convention".to_owned());
    let mut config = use_signal(|| Value::Object(Default::default()));
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let descriptors = compiler_extensions.descriptors();
    let selected_extension = ExtensionType::from_provider_key(renderer());
    let submit_extensions = compiler_extensions.clone();
    let select_extensions = compiler_extensions.clone();

    rsx! {
        Dialog {
            open: open(),
            on_open_change: move |value| open.set(value),
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let page_name = name().trim().to_owned();
                    let page_title = title().trim().to_owned();
                    if page_name.is_empty() || page_title.is_empty() {
                        error.set(Some("菜单标识和标题不能为空".to_owned()));
                        return;
                    }
                    let page_id = DefinitionId::new();
                    let selected = renderer();
                    let page_renderer = if selected == "convention" {
                        PageRendererDefinition::ConventionFile
                    } else {
                        let extension_type = ExtensionType::from_provider_key(selected);
                        let Some(extension) = submit_extensions.get(&extension_type) else {
                            error.set(Some(format!("页面扩展未注册: {extension_type}")));
                            return;
                        };
                        PageRendererDefinition::Extension {
                            extension_type,
                            schema_version: extension.schema_version(),
                            config: config(),
                        }
                    };
                    let command = AdminCommand::AddMenuPage {
                        scene_id: scene_id.clone(),
                        parent_menu_id: None,
                        menu: MenuDefinition {
                            id: DefinitionId::new(),
                            name: page_name.clone(),
                            title: page_title.clone(),
                            icon: Some(icon()),
                            page_id: Some(page_id.clone()),
                            enabled: true,
                            children: Vec::new(),
                        },
                        page: PageDefinition {
                            id: page_id.clone(),
                            name: page_name,
                            title: page_title.clone(),
                            renderer: page_renderer,
                        },
                    };
                    let provider = admin.provider().clone();
                    pending.set(true);
                    spawn(async move {
                        match provider.execute(command).await {
                            Ok(_) => {
                                if renderer() == "convention" {
                                    match provider.generate_convention_file(page_id).await {
                                        Ok(result) => on_status.call(format!(
                                            "已创建菜单并{}约定文件：{}",
                                            if result.created { "生成" } else { "复用" },
                                            result.path
                                        )),
                                        Err(cause) => on_status.call(format!(
                                            "菜单已创建，约定文件生成失败：{cause}"
                                        )),
                                    }
                                } else {
                                    on_status.call(format!("已创建扩展页面：{page_title}"));
                                }
                                generation.with_mut(|value| *value = value.saturating_add(1));
                                open.set(false);
                            }
                            Err(cause) => error.set(Some(cause.to_string())),
                        }
                        pending.set(false);
                    });
                },
                header {
                    DialogTitle { "新建菜单页面" }
                    DialogDescription { "选择消费方约定文件，或由已注册扩展提供页面能力。" }
                }
                label { r#for: "menu-name", "菜单标识" }
                Input {
                    id: "menu-name",
                    name: "name",
                    value: "{name}",
                    oninput: move |event: FormEvent| name.set(event.value()),
                }
                label { r#for: "menu-title", "菜单标题" }
                Input {
                    id: "menu-title",
                    name: "title",
                    value: "{title}",
                    oninput: move |event: FormEvent| title.set(event.value()),
                }
                label { "菜单图标" }
                NavigationIconPicker {
                    name: "icon",
                    value: icon,
                    aria_label: "菜单图标",
                    on_value_change: move |value| icon.set(value),
                }
                label { r#for: "page-renderer", "页面来源" }
                select {
                    id: "page-renderer",
                    name: "renderer",
                    value: "{renderer}",
                    onchange: move |event: FormEvent| {
                        let next = event.value();
                        if next == "convention" {
                            config.set(Value::Object(Default::default()));
                        } else {
                            let extension_type = ExtensionType::from_provider_key(next.clone());
                            if let Some(extension) = select_extensions.get(&extension_type) {
                                config.set(extension.default_config());
                            }
                        }
                        renderer.set(next);
                    },
                    option { value: "convention", "约定文件" }
                    for descriptor in &descriptors {
                        option {
                            value: "{descriptor.extension_type}",
                            "{descriptor.title}"
                        }
                    }
                }
                if renderer() != "convention" {
                    if let Some(extension) = renderer_extensions.get(&selected_extension) {
                        {extension.render_editor(PageExtensionEditorContext {
                            page_id: DefinitionId::new(),
                            config: config(),
                            resources: resources.clone(),
                            on_change: Callback::new(move |value| config.set(value)),
                        })}
                    } else {
                        p { role: "alert", "扩展缺少 Dioxus 编辑器：{selected_extension}" }
                    }
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
                    Button { r#type: "submit", disabled: pending(), "创建菜单" }
                }
            }
        }
    }
}
