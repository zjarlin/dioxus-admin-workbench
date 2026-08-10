use std::collections::BTreeSet;

use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronRight;

use super::data::{CollectionTreeData, resolve_visible_items};
use crate::button::{Button, ButtonSize, ButtonVariant};

#[css_module("/src/collection_tree/style.css")]
struct Styles;

pub(crate) fn load_stylesheet() {
    drop(Styles::collection_tree.to_string());
}

#[derive(Clone, PartialEq)]
pub struct CollectionTreeItemContext<T: Clone + PartialEq + 'static> {
    pub item: T,
    pub key: String,
    pub depth: usize,
    pub selected: bool,
    pub has_children: bool,
    pub expanded: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct CollectionTreeProps<T: Clone + PartialEq + 'static> {
    pub aria_label: String,
    pub data: CollectionTreeData<T>,
    pub item_key: Callback<T, String>,
    pub render_item: Callback<CollectionTreeItemContext<T>, Element>,
    #[props(default)]
    pub selected_key: Option<String>,
    #[props(default)]
    pub on_select: Option<EventHandler<T>>,
    #[props(default = "暂无数据".to_owned())]
    pub empty_text: String,
    #[props(default)]
    pub class: String,
}

#[component]
pub fn CollectionTree<T: Clone + PartialEq + 'static>(props: CollectionTreeProps<T>) -> Element {
    let mut collapsed_keys = use_signal(BTreeSet::<String>::new);
    let is_tree = props.data.is_tree();
    let rows = match resolve_visible_items(&props.data, &collapsed_keys(), |item| {
        props.item_key.call(item.clone())
    }) {
        Ok(rows) => rows,
        Err(error) => return collection_tree_error(&error),
    };
    let root_class = classes(
        &Styles::collection_tree,
        (!props.class.is_empty()).then_some(props.class.as_str()),
    );
    let root_role = if is_tree { "tree" } else { "listbox" };
    let item_role = if is_tree { "treeitem" } else { "option" };

    rsx! {
        div { class: root_class, role: root_role, aria_label: props.aria_label,
            if props.data.is_empty() {
                div { class: Styles::collection_tree_empty, "{props.empty_text}" }
            }
            for row in rows {
                {
                    let selected = props.selected_key.as_deref() == Some(row.key.as_str());
                    let row_class = classes(
                        &Styles::collection_tree_row,
                        selected
                            .then_some(Styles::collection_tree_row_selected)
                            .as_deref(),
                    );
                    let row_style = format!("--collection-tree-depth:{};", row.depth);
                    let key_for_toggle = row.key.clone();
                    let item_for_select = row.item.clone();
                    let item_context = CollectionTreeItemContext {
                        item: row.item,
                        key: row.key.clone(),
                        depth: row.depth,
                        selected,
                        has_children: row.has_children,
                        expanded: row.expanded,
                    };
                    rsx! {
                        div {
                            key: "{row.key}",
                            class: row_class,
                            style: row_style,
                            role: item_role,
                            "data-mode": if is_tree { "tree" } else { "collection" },
                            aria_selected: selected.to_string(),
                            aria_level: is_tree.then(|| (row.depth + 1).to_string()),
                            aria_expanded: row.has_children.then(|| row.expanded.to_string()),
                            if is_tree {
                                if row.has_children {
                                    Button {
                                        r#type: "button",
                                        class: Styles::collection_tree_toggle,
                                        size: ButtonSize::IconXs,
                                        variant: ButtonVariant::Ghost,
                                        title: if row.expanded { "收起子项" } else { "展开子项" },
                                        aria_label: if row.expanded { "收起子项" } else { "展开子项" },
                                        "data-state": if row.expanded { "expanded" } else { "collapsed" },
                                        onclick: move |_| {
                                            let mut keys = collapsed_keys.write();
                                            if !keys.insert(key_for_toggle.clone()) {
                                                keys.remove(&key_for_toggle);
                                            }
                                        },
                                        ChevronRight { class: "size-3" }
                                    }
                                } else {
                                    span { class: Styles::collection_tree_spacer, aria_hidden: "true" }
                                }
                            }
                            Button {
                                r#type: "button",
                                class: Styles::collection_tree_item,
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| {
                                    if let Some(handler) = props.on_select {
                                        handler.call(item_for_select.clone());
                                    }
                                },
                                {props.render_item.call(item_context)}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn collection_tree_error(error: &str) -> Element {
    rsx! {
        div { class: Styles::collection_tree_error, role: "alert", "集合树配置错误：{error}" }
    }
}

fn classes(base: &str, extra: Option<&str>) -> String {
    match extra {
        Some(extra) => format!("{base} {extra}"),
        None => base.to_owned(),
    }
}
