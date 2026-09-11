use super::{ListState, SortValue};
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    checkbox::{Checkbox, CheckboxState},
    data_table::{DataTable, DataTableCellContext, DataTableColumn, DataTableHeaderContext},
    input::Input,
    select::{Select, SelectItem},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowDown, ArrowUp, ChevronLeft, ChevronRight, ChevronsUpDown, Search, X,
};
use std::collections::BTreeSet;

#[component]
pub fn CollectionTable<R: Clone + PartialEq + 'static>(
    label: String,
    rows: Vec<R>,
    columns: Vec<DataTableColumn>,
    row_key: Callback<R, String>,
    search_text: Callback<R, String>,
    sort_value: Callback<(R, String), SortValue>,
    sortable: Vec<String>,
    render_cell: Callback<DataTableCellContext<R>, Element>,
    #[props(default)] on_selection_change: Option<Callback<Vec<R>>>,
    #[props(default)] selected_keys: ReadSignal<Option<BTreeSet<String>>>,
    #[props(default = rsx! {})] tools: Element,
    #[props(default = "暂无数据".to_owned())] empty_text: String,
) -> Element {
    let mut state = use_signal(ListState::default);
    let mut selection = use_signal(BTreeSet::<String>::new);
    let current_selection = use_memo(move || selected_keys().unwrap_or_else(|| selection()));
    let query = state();
    let filtered = query.query(
        &rows,
        |row| search_text.call(row.clone()),
        |row, key| sort_value.call((row.clone(), key.into())),
    );
    let total = filtered.len();
    let range = query.range(total);
    let page = query.page.min(query.page_count(total) - 1);
    let visible = filtered[range.clone()].to_vec();
    let keys = visible
        .iter()
        .map(|row| row_key.call(row.clone()))
        .collect::<BTreeSet<_>>();
    let selected = current_selection();
    let all = !keys.is_empty() && keys.is_subset(&selected);
    let some = keys.iter().any(|key| selected.contains(key));
    let selected_count = rows
        .iter()
        .filter(|row| selected.contains(&row_key.call((*row).clone())))
        .count();
    let notify = use_callback(move |keys: BTreeSet<String>| {
        selection.set(keys.clone());
        if let Some(callback) = on_selection_change {
            callback.call(
                rows.iter()
                    .filter(|row| keys.contains(&row_key.call((*row).clone())))
                    .cloned()
                    .collect(),
            );
        }
    });
    let mut displayed_columns = columns;
    if on_selection_change.is_some() {
        displayed_columns.insert(0, DataTableColumn::leaf("__selection", "选择").width(48));
    }
    let lower = if total == 0 { 0 } else { range.start + 1 };
    let upper = range.end;
    rsx! {
        div { class: "admin-collection",
            div { class: "admin-toolbar",
                div { class: "admin-search-field",
                    label { class: "admin-search-label",
                        span { "搜索{label}" }
                        div { class: "admin-search",
                            Search { class: "admin-search-icon" }
                            Input { r#type: "search", aria_label: "搜索{label}", placeholder: "名称或关键词", value: query.search.clone(),
                                oninput: move |event: FormEvent| state.write().search(event.value()),
                            }
                        }
                    }
                    if !query.search.is_empty() {
                        Button { class: "admin-search-clear", size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, aria_label: "清除搜索", title: "清除搜索", onclick: move |_| state.write().search(String::new()), X {} }
                    }
                }
                div { class: "admin-actions", {tools} }
            }
            if selected_count > 0 {
                div { class: "admin-selection", role: "status",
                    span { "已选择 {selected_count} 项" }
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| notify.call(BTreeSet::new()), "取消选择" }
                }
            }
            DataTable {
                aria_label: label.clone(), rows: visible, columns: displayed_columns, row_key,
                class: "admin-table", empty_text: if total == 0 && !query.search.is_empty() { "没有匹配结果".to_owned() } else { empty_text },
                sort_column: if query.sort_key.is_empty() { None } else { Some(query.sort_key.clone()) },
                sort_descending: query.descending,
                render_header: move |header: DataTableHeaderContext| {
                    if header.column.key == "__selection" {
                        let keys = keys.clone();
                        return rsx! { Checkbox { aria_label: "选择当前页", checked: Some(if all { CheckboxState::Checked } else if some { CheckboxState::Indeterminate } else { CheckboxState::Unchecked }),
                            disabled: keys.is_empty(),
                            on_checked_change: move |_: CheckboxState| {
                                let mut next = current_selection();
                                if !all { next.extend(keys.iter().cloned()); } else { next.retain(|key| !keys.contains(key)); }
                                notify.call(next);
                            }
                        } };
                    }
                    if !sortable.contains(&header.column.key) { return rsx! { "{header.column.title}" }; }
                    let current = state();
                    let active = current.sort_key == header.column.key;
                    rsx! { button { class: "admin-sort", r#type: "button", aria_label: "排序：{header.column.title}", onclick: move |_| state.write().sort(header.column.key.clone()),
                        "{header.column.title}"
                        if active && current.descending { ArrowDown {} } else if active { ArrowUp {} } else { ChevronsUpDown {} }
                    } }
                },
                render_cell: move |cell: DataTableCellContext<R>| {
                    if cell.column.key == "__selection" {
                        let key = row_key.call(cell.row.clone());
                        let name = search_text.call(cell.row);
                        rsx! { Checkbox { aria_label: "选择 {name}", checked: Some(if current_selection().contains(&key) { CheckboxState::Checked } else { CheckboxState::Unchecked }),
                            on_checked_change: move |checked: CheckboxState| {
                                let mut next = current_selection();
                                if bool::from(checked) { next.insert(key.clone()); } else { next.remove(&key); }
                                notify.call(next);
                            }
                        } }
                    } else { render_cell.call(cell) }
                },
            }
            footer { class: "admin-pagination",
                p { role: "status", "第 {lower}–{upper} 项，共 {total} 项" }
                div { class: "admin-actions",
                    Select { aria_label: "每页数量", value: query.page_size.to_string(),
                        options: [10, 20, 50].into_iter().map(|size| SelectItem::new(size.to_string(), format!("{size} 条 / 页"))).collect(),
                        on_value_change: move |value: String| { let mut query = state.write(); query.page_size = value.parse().unwrap_or(10); query.page = 0; },
                    }
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Icon, aria_label: "上一页", title: "上一页", disabled: page == 0, onclick: move |_| state.write().page = page.saturating_sub(1), ChevronLeft {} }
                    span { class: "admin-page-number", "{page + 1} / {query.page_count(total)}" }
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Icon, aria_label: "下一页", title: "下一页", disabled: page + 1 >= query.page_count(total), onclick: move |_| state.write().page = page + 1, ChevronRight {} }
                }
            }
        }
    }
}
