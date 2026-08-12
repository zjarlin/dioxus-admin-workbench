use az_admin_shell_core::{DefinitionId, MenuDefinition};
use az_ui_components::navigation_icon::{NavigationIcon, resolved_navigation_icon};
use dioxus::prelude::*;

pub(crate) fn first_page(menus: &[MenuDefinition]) -> Option<DefinitionId> {
    menus
        .iter()
        .find_map(|menu| menu.page_id.clone().or_else(|| first_page(&menu.children)))
}

pub(crate) fn menu_items(
    menus: &[MenuDefinition],
    mut selected_page: Signal<Option<DefinitionId>>,
    row_class: String,
    active_class: String,
    children_class: String,
) -> Element {
    rsx! {
        for menu in menus.iter().filter(|menu| menu.enabled) {
            div { key: "{menu.id}",
                if let Some(page_id) = menu.page_id.clone() {
                    button {
                        class: if selected_page() == Some(page_id.clone()) {
                            format!("{row_class} {active_class}")
                        } else {
                            row_class.clone()
                        },
                        r#type: "button",
                        title: menu.title.clone(),
                        aria_label: menu.title.clone(),
                        onclick: move |_| selected_page.set(Some(page_id.clone())),
                        span { aria_hidden: "true",
                            NavigationIcon {
                                name: menu_icon(menu).to_owned(),
                                class: "size-4".to_owned(),
                            }
                        }
                        span { "{menu.title}" }
                    }
                } else {
                    div { class: row_class.clone(), "{menu.title}" }
                }
                if !menu.children.is_empty() {
                    div { class: children_class.clone(),
                        {menu_items(
                            &menu.children,
                            selected_page,
                            row_class.clone(),
                            active_class.clone(),
                            children_class.clone(),
                        )}
                    }
                }
            }
        }
    }
}

fn menu_icon(menu: &MenuDefinition) -> &'static str {
    resolved_navigation_icon(menu.icon.as_deref(), &menu.name)
}
