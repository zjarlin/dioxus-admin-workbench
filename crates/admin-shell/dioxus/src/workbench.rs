use std::sync::Arc;

use az_admin_shell_core::{DefinitionId, PageExtensionCompilerIndex};
use az_ui_components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use icons::{PanelLeft, Plus};

use crate::{
    AdminProviderHandle, ConventionPageIndex, PageExtensionRendererIndex,
    menu_dialog::MenuDialog,
    navigation::{first_page, menu_items},
    provider::resolve_admin_provider,
    runtime::{error_state, render_page},
    scene_dialog::SceneDialog,
};

#[css_module("/src/style.css")]
struct Styles;

#[allow(non_snake_case)]
pub fn App() -> Element {
    let resolved = use_hook(|| {
        let mut context = rudi::Context::auto_register();
        let admin = resolve_admin_provider(&mut context).map_err(|error| error.to_string())?;
        let compiler_extensions = PageExtensionCompilerIndex::from_context(&mut context)
            .map_err(|error| error.to_string())?;
        let renderer_extensions = PageExtensionRendererIndex::from_context(&mut context)
            .map_err(|error| error.to_string())?;
        let convention_pages =
            ConventionPageIndex::from_context(&mut context).map_err(|error| error.to_string())?;
        Ok::<_, String>((
            admin,
            Arc::new(compiler_extensions),
            Arc::new(renderer_extensions),
            Arc::new(convention_pages),
        ))
    });
    let Ok((admin, compiler_extensions, renderer_extensions, convention_pages)) = resolved else {
        return error_state(
            "工作台初始化失败",
            &resolved
                .as_ref()
                .err()
                .map(ToString::to_string)
                .unwrap_or_default(),
        );
    };
    rsx! {
        AdminShell {
            admin: admin.clone(),
            compiler_extensions: compiler_extensions.clone(),
            renderer_extensions: renderer_extensions.clone(),
            convention_pages: convention_pages.clone(),
        }
    }
}

#[component]
pub fn AdminShell(
    admin: AdminProviderHandle,
    compiler_extensions: Arc<PageExtensionCompilerIndex>,
    renderer_extensions: Arc<PageExtensionRendererIndex>,
    convention_pages: Arc<ConventionPageIndex>,
) -> Element {
    az_ui_components::load_deferred_stylesheets();
    let generation = use_signal(|| 0_u64);
    let mut selected_scene = use_signal(|| None::<DefinitionId>);
    let mut selected_page = use_signal(|| None::<DefinitionId>);
    let mut sidebar_collapsed = use_signal(|| false);
    let mut scene_dialog_open = use_signal(|| false);
    let mut menu_dialog_open = use_signal(|| false);
    let mut status = use_signal(|| None::<String>);
    let resource_admin = admin.clone();
    let snapshot = use_resource(move || {
        let _generation = generation();
        let provider = resource_admin.provider().clone();
        async move { provider.load().await.map_err(|cause| cause.to_string()) }
    });
    let loaded = snapshot.read().as_ref().cloned();
    let Some(loaded) = loaded else {
        return rsx! { div { class: Styles::state, "正在加载工作台..." } };
    };
    let Ok(snapshot) = loaded else {
        return error_state(
            "加载工作台失败",
            loaded
                .as_ref()
                .err()
                .map(String::as_str)
                .unwrap_or_default(),
        );
    };

    let active_scene_id = selected_scene()
        .filter(|id| snapshot.definition.scene(id).is_some())
        .or_else(|| {
            snapshot
                .definition
                .scenes
                .first()
                .map(|scene| scene.id.clone())
        });
    if selected_scene() != active_scene_id {
        selected_scene.set(active_scene_id.clone());
    }
    let active_scene = active_scene_id
        .as_ref()
        .and_then(|id| snapshot.definition.scene(id));
    let active_page_id = selected_page()
        .filter(|id| snapshot.compiled.pages.contains_key(id))
        .or_else(|| active_scene.and_then(|scene| first_page(&scene.menus)));
    if selected_page() != active_page_id {
        selected_page.set(active_page_id.clone());
    }
    let content = active_page_id
        .as_ref()
        .and_then(|id| snapshot.compiled.pages.get(id))
        .cloned()
        .map(|page| {
            render_page(
                page,
                snapshot.resources.clone(),
                admin.clone(),
                &renderer_extensions,
                &convention_pages,
            )
        })
        .unwrap_or_else(|| rsx! { div { class: Styles::state, "请选择菜单页面" } });

    rsx! {
        div {
            class: Styles::shell,
            "data-sidebar-collapsed": sidebar_collapsed().to_string(),
            aside { class: Styles::sidebar,
                header { class: Styles::sidebar_header,
                    Button {
                        size: ButtonSize::IconSm,
                        variant: ButtonVariant::Ghost,
                        title: if sidebar_collapsed() { "展开侧栏" } else { "收起侧栏" },
                        aria_label: if sidebar_collapsed() { "展开侧栏" } else { "收起侧栏" },
                        onclick: move |_| sidebar_collapsed.toggle(),
                        PanelLeft { class: "size-4" }
                    }
                    strong { class: Styles::brand, "{snapshot.definition.title}" }
                }
                nav { class: Styles::menu, aria_label: "页面菜单",
                    if let Some(scene) = active_scene {
                        {menu_items(
                            &scene.menus,
                            selected_page,
                            Styles::menu_row.to_string(),
                            Styles::menu_row_active.to_string(),
                            Styles::menu_children.to_string(),
                        )}
                    }
                }
                footer { class: Styles::sidebar_footer,
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        disabled: active_scene_id.is_none(),
                        title: "新建菜单",
                        aria_label: "新建菜单",
                        onclick: move |_| menu_dialog_open.set(true),
                        Plus { class: "size-4" }
                        span { class: Styles::action_label, "新建菜单" }
                    }
                }
            }
            main { class: Styles::main,
                header { class: Styles::topbar,
                    h1 {
                        {active_page_id
                            .as_ref()
                            .and_then(|id| snapshot.compiled.pages.get(id))
                            .map_or(snapshot.definition.title.as_str(), |page| page.title.as_str())}
                    }
                    nav { class: Styles::scenes, aria_label: "场景",
                        for scene in &snapshot.definition.scenes {
                            Button {
                                key: "{scene.id}",
                                size: ButtonSize::Sm,
                                variant: if active_scene_id.as_ref() == Some(&scene.id) {
                                    ButtonVariant::Secondary
                                } else {
                                    ButtonVariant::Ghost
                                },
                                onclick: {
                                    let scene_id = scene.id.clone();
                                    move |_| {
                                        selected_scene.set(Some(scene_id.clone()));
                                        selected_page.set(None);
                                    }
                                },
                                "{scene.title}"
                            }
                        }
                        Button {
                            size: ButtonSize::IconSm,
                            variant: ButtonVariant::Outline,
                            title: "新建场景",
                            aria_label: "新建场景",
                            onclick: move |_| scene_dialog_open.set(true),
                            Plus { class: "size-4" }
                        }
                    }
                }
                if let Some(message) = status() {
                    div { class: Styles::status, role: "status", "{message}" }
                }
                section { class: Styles::content, {content} }
            }
            if scene_dialog_open() {
                SceneDialog {
                    admin: admin.clone(),
                    open: scene_dialog_open,
                    generation,
                    on_status: Callback::new(move |message| status.set(Some(message))),
                }
            }
            if menu_dialog_open() {
                if let Some(scene_id) = active_scene_id {
                    MenuDialog {
                        admin: admin.clone(),
                        scene_id,
                        resources: snapshot.resources.clone(),
                        compiler_extensions: compiler_extensions.clone(),
                        renderer_extensions: renderer_extensions.clone(),
                        open: menu_dialog_open,
                        generation,
                        on_status: Callback::new(move |message| status.set(Some(message))),
                    }
                }
            }
        }
    }
}
