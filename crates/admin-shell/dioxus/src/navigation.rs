use az_admin_shell_core::{DefinitionId, MenuDefinition};
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
                        span { aria_hidden: "true", "{menu_icon(menu)}" }
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

fn menu_icon(menu: &MenuDefinition) -> String {
    menu.icon
        .clone()
        .or_else(|| {
            menu.title
                .chars()
                .next()
                .map(|character| character.to_string())
        })
        .unwrap_or_default()
}
