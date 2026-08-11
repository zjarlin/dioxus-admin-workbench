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

const SHELL_STYLESHEET: Asset = asset!("/src/style.css", AssetOptions::css());
const SHELL_CLASS: &str = "admin-shell";
const SIDEBAR_CLASS: &str = "admin-shell__sidebar";
const SIDEBAR_HEADER_CLASS: &str = "admin-shell__sidebar-header";
const SIDEBAR_FOOTER_CLASS: &str = "admin-shell__sidebar-footer";
const BRAND_CLASS: &str = "admin-shell__brand";
const ACTION_LABEL_CLASS: &str = "admin-shell__action-label";
const MENU_CLASS: &str = "admin-shell__menu";
const MENU_ROW_CLASS: &str = "admin-shell__menu-row";
const MENU_ROW_ACTIVE_CLASS: &str = "admin-shell__menu-row--active";
const MENU_CHILDREN_CLASS: &str = "admin-shell__menu-children";
const MAIN_CLASS: &str = "admin-shell__main";
const TOPBAR_CLASS: &str = "admin-shell__topbar";
const SCENES_CLASS: &str = "admin-shell__scenes";
const STATUS_CLASS: &str = "admin-shell__status";
const CONTENT_CLASS: &str = "admin-shell__content";
const STATE_CLASS: &str = "admin-shell__state";

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
        return styled_shell(rsx! { div { class: STATE_CLASS, "正在加载工作台..." } });
    };
    let Ok(snapshot) = loaded else {
        return styled_shell(error_state(
            "加载工作台失败",
            loaded
                .as_ref()
                .err()
                .map(String::as_str)
                .unwrap_or_default(),
        ));
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
        .unwrap_or_else(|| rsx! { div { class: STATE_CLASS, "请选择菜单页面" } });

    styled_shell(rsx! {
        div {
            class: SHELL_CLASS,
            "data-sidebar-collapsed": sidebar_collapsed().to_string(),
            aside { class: SIDEBAR_CLASS,
                header { class: SIDEBAR_HEADER_CLASS,
                    Button {
                        size: ButtonSize::IconSm,
                        variant: ButtonVariant::Ghost,
                        title: if sidebar_collapsed() { "展开侧栏" } else { "收起侧栏" },
                        aria_label: if sidebar_collapsed() { "展开侧栏" } else { "收起侧栏" },
                        onclick: move |_| sidebar_collapsed.toggle(),
                        PanelLeft { class: "size-4" }
                    }
                    strong { class: BRAND_CLASS, "{snapshot.definition.title}" }
                }
                nav { class: MENU_CLASS, aria_label: "页面菜单",
                    if let Some(scene) = active_scene {
                        {menu_items(
                            &scene.menus,
                            selected_page,
                            MENU_ROW_CLASS.to_owned(),
                            MENU_ROW_ACTIVE_CLASS.to_owned(),
                            MENU_CHILDREN_CLASS.to_owned(),
                        )}
                    }
                }
                footer { class: SIDEBAR_FOOTER_CLASS,
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        disabled: active_scene_id.is_none(),
                        title: "新建菜单",
                        aria_label: "新建菜单",
                        onclick: move |_| menu_dialog_open.set(true),
                        Plus { class: "size-4" }
                        span { class: ACTION_LABEL_CLASS, "新建菜单" }
                    }
                }
            }
            main { class: MAIN_CLASS,
                header { class: TOPBAR_CLASS,
                    h1 {
                        {active_page_id
                            .as_ref()
                            .and_then(|id| snapshot.compiled.pages.get(id))
                            .map_or(snapshot.definition.title.as_str(), |page| page.title.as_str())}
                    }
                    nav { class: SCENES_CLASS, aria_label: "场景",
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
                    div { class: STATUS_CLASS, role: "status", "{message}" }
                }
                section { class: CONTENT_CLASS, {content} }
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
    })
}

fn styled_shell(content: Element) -> Element {
    rsx! {
        az_ui_components::UiStylesheets {}
        document::Stylesheet { href: SHELL_STYLESHEET }
        {content}
    }
}

#[cfg(test)]
mod tests {
    use super::{MENU_ROW_ACTIVE_CLASS, SHELL_CLASS};

    #[test]
    fn shell_stylesheet_keeps_stable_domain_classes() {
        let stylesheet = include_str!("style.css");

        assert!(stylesheet.contains(&format!(".{SHELL_CLASS}")));
        assert!(stylesheet.contains(&format!(".{MENU_ROW_ACTIVE_CLASS}")));
    }
}
