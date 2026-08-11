use dioxus::prelude::*;

use super::layout::{
    DataTableAlign, DataTableBodyCell, DataTableColumn, DataTableFixed, DataTableSpan,
    ResolvedLeafColumn, resolve_body_grid, resolve_header_rows, resolve_leaf_columns,
};

const TABLE_CLASS: &str = "data-table";
const TABLE_ALIGN_CENTER_CLASS: &str = "data-table-align-center";
const TABLE_ALIGN_END_CLASS: &str = "data-table-align-end";
const TABLE_ALIGN_START_CLASS: &str = "data-table-align-start";
const TABLE_CELL_CLASS: &str = "data-table-cell";
const TABLE_CELL_EDITABLE_CLASS: &str = "data-table-cell-editable";
const TABLE_CELL_EDITING_CLASS: &str = "data-table-cell-editing";
const TABLE_EMPTY_CLASS: &str = "data-table-empty";
const TABLE_ERROR_CLASS: &str = "data-table-error";
const TABLE_FIXED_LEFT_CLASS: &str = "data-table-fixed-left";
const TABLE_FIXED_RIGHT_CLASS: &str = "data-table-fixed-right";
const TABLE_HEADER_CELL_CLASS: &str = "data-table-header-cell";
const TABLE_HEADER_GROUP_CLASS: &str = "data-table-header-group";
const TABLE_PANEL_CLASS: &str = "data-table-panel";
const TABLE_ROOT_CLASS: &str = "data-table-root";
const TABLE_ROW_CLASS: &str = "data-table-row";
const TABLE_ROW_SELECTED_CLASS: &str = "data-table-row-selected";
const TABLE_VIEWPORT_CLASS: &str = "data-table-viewport";
const TABLE_WORKSPACE_CLASS: &str = "data-table-workspace";
const TABLE_WORKSPACE_WITH_PANEL_CLASS: &str = "data-table-workspace-with-panel";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DataTableEditTrigger {
    Click,
    #[default]
    DoubleClick,
    Manual,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DataTableCellAddress {
    row_key: String,
    column_key: String,
}

#[derive(Clone, PartialEq)]
pub struct DataTableCellContext<R: Clone + PartialEq + 'static> {
    pub row: R,
    pub row_index: usize,
    pub row_key: String,
    pub column: DataTableColumn,
}

#[derive(Clone, PartialEq)]
pub struct DataTableEditContext<R: Clone + PartialEq + 'static> {
    pub cell: DataTableCellContext<R>,
    pub close: Callback<()>,
}

#[derive(Clone, PartialEq)]
pub struct DataTableHeaderContext {
    pub column: DataTableColumn,
}

#[derive(Props, Clone, PartialEq)]
pub struct DataTableProps<R: Clone + PartialEq + 'static> {
    pub aria_label: String,
    pub rows: Vec<R>,
    pub columns: Vec<DataTableColumn>,
    pub row_key: Callback<R, String>,
    pub render_cell: Callback<DataTableCellContext<R>, Element>,
    #[props(default)]
    pub render_header: Option<Callback<DataTableHeaderContext, Element>>,
    #[props(default)]
    pub render_editor: Option<Callback<DataTableEditContext<R>, Element>>,
    #[props(default)]
    pub can_edit: Option<Callback<DataTableCellContext<R>, bool>>,
    #[props(default)]
    pub spans: Vec<DataTableSpan>,
    #[props(default)]
    pub selected_row_key: Option<String>,
    #[props(default)]
    pub on_row_select: Option<EventHandler<R>>,
    #[props(default)]
    pub right_panel: Option<Element>,
    #[props(default = "32rem".to_owned())]
    pub max_height: String,
    #[props(default = 360)]
    pub right_panel_width: u16,
    #[props(default = true)]
    pub sticky_header: bool,
    #[props(default)]
    pub edit_trigger: DataTableEditTrigger,
    #[props(default = "暂无数据".to_owned())]
    pub empty_text: String,
    #[props(default)]
    pub class: String,
}

#[component]
pub fn DataTable<R: Clone + PartialEq + 'static>(props: DataTableProps<R>) -> Element {
    let mut active_cell = use_signal(|| None::<DataTableCellAddress>);
    let close_editor = use_callback(move |_: ()| active_cell.set(None));
    let leaves = match resolve_leaf_columns(&props.columns) {
        Ok(leaves) => leaves,
        Err(error) => return table_error(&error),
    };
    let header_rows = match resolve_header_rows(&props.columns, &leaves) {
        Ok(rows) => rows,
        Err(error) => return table_error(&error),
    };
    let body_grid = match resolve_body_grid(props.rows.len(), &leaves, &props.spans) {
        Ok(grid) => grid,
        Err(error) => return table_error(&error),
    };
    let table_width = leaves
        .iter()
        .map(|leaf| u32::from(leaf.column.width))
        .sum::<u32>();
    let has_right_panel = props.right_panel.is_some();
    let workspace_class = classes(
        TABLE_WORKSPACE_CLASS,
        has_right_panel.then_some(TABLE_WORKSPACE_WITH_PANEL_CLASS),
    );
    let root_class = classes(
        TABLE_ROOT_CLASS,
        (!props.class.is_empty()).then_some(props.class.as_str()),
    );
    let root_style = format!(
        "--data-table-max-height:{};--data-table-panel-width:{}px;",
        props.max_height, props.right_panel_width
    );
    let table_style = format!("min-width:{table_width}px;");
    let right_panel = props.right_panel.clone();

    rsx! {
        div { class: root_class, style: root_style,
            div { class: workspace_class,
                div { class: TABLE_VIEWPORT_CLASS,
                    table {
                        class: TABLE_CLASS,
                        style: table_style,
                        aria_label: props.aria_label,
                        "data-sticky-header": props.sticky_header.to_string(),
                        colgroup {
                            for leaf in &leaves {
                                col { style: format!("width:{}px;", leaf.column.width) }
                            }
                        }
                        thead {
                            for (row_index, row) in header_rows.iter().enumerate() {
                                tr { key: "header-{row_index}",
                                    for header in row {
                                        th {
                                            key: "{header.key}",
                                            class: header_class(header.fixed, header.leaf_key.is_none()),
                                            style: header_style(header),
                                            colspan: header.colspan,
                                            rowspan: header.rowspan,
                                            scope: if header.leaf_key.is_some() { "col" } else { "colgroup" },
                                            "data-column-key": header.key.clone(),
                                            "data-header-level": header.level,
                                            {render_header_content(
                                                header,
                                                &leaves,
                                                props.render_header,
                                            )}
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            for (row_index, row) in props.rows.iter().enumerate() {
                                {render_row(
                                    row.clone(),
                                    row_index,
                                    &leaves,
                                    &body_grid[row_index],
                                    &props,
                                    active_cell,
                                    close_editor,
                                )}
                            }
                        }
                    }
                    if props.rows.is_empty() {
                        div { class: TABLE_EMPTY_CLASS, "{props.empty_text}" }
                    }
                }
                if let Some(panel) = right_panel {
                    aside { class: TABLE_PANEL_CLASS, aria_label: "表格编辑区",
                        {panel}
                    }
                }
            }
        }
    }
}

fn render_row<R: Clone + PartialEq + 'static>(
    row: R,
    row_index: usize,
    leaves: &[ResolvedLeafColumn],
    layout: &[DataTableBodyCell],
    props: &DataTableProps<R>,
    active_cell: Signal<Option<DataTableCellAddress>>,
    close_editor: Callback<()>,
) -> Element {
    let row_key = props.row_key.call(row.clone());
    let selected = props.selected_row_key.as_deref() == Some(row_key.as_str());
    let row_class = classes(
        TABLE_ROW_CLASS,
        selected.then_some(TABLE_ROW_SELECTED_CLASS),
    );
    let row_for_select = row.clone();
    let on_row_select = props.on_row_select;

    rsx! {
        tr {
            key: "{row_key}",
            class: row_class,
            aria_selected: selected.to_string(),
            "data-row-key": row_key.clone(),
            onclick: move |_| {
                if let Some(handler) = on_row_select {
                    handler.call(row_for_select.clone());
                }
            },
            for (column_index, leaf) in leaves.iter().enumerate() {
                if layout[column_index].visible {
                                    {render_cell(
                                        row.clone(),
                                        row_index,
                                        row_key.clone(),
                                        column_index,
                                        leaves,
                                        leaf,
                                        layout[column_index],
                        props,
                        active_cell,
                        close_editor,
                    )}
                }
            }
        }
    }
}

fn render_cell<R: Clone + PartialEq + 'static>(
    row: R,
    row_index: usize,
    row_key: String,
    column_index: usize,
    leaves: &[ResolvedLeafColumn],
    leaf: &ResolvedLeafColumn,
    layout: DataTableBodyCell,
    props: &DataTableProps<R>,
    mut active_cell: Signal<Option<DataTableCellAddress>>,
    close_editor: Callback<()>,
) -> Element {
    let context = DataTableCellContext {
        row,
        row_index,
        row_key: row_key.clone(),
        column: leaf.column.clone(),
    };
    let editable = leaf.column.editable
        && props.render_editor.is_some()
        && props
            .can_edit
            .map(|can_edit| can_edit.call(context.clone()))
            .unwrap_or(true);
    let address = DataTableCellAddress {
        row_key,
        column_key: leaf.column.key.clone(),
    };
    let editing = active_cell().as_ref() == Some(&address);
    let editor_address = address.clone();
    let double_click_address = address.clone();
    let keyboard_address = address.clone();
    let trigger = props.edit_trigger;
    let class = body_cell_class(leaf, editable, editing);
    let width = leaves[column_index..column_index + layout.colspan]
        .iter()
        .map(|column| u32::from(column.column.width))
        .sum();
    let style = body_cell_style(leaf, layout, width);

    rsx! {
        td {
            class,
            style,
            rowspan: layout.rowspan,
            colspan: layout.colspan,
            tabindex: if editable { "0" } else { "-1" },
            "data-column-key": leaf.column.key.clone(),
            "data-editable": editable.to_string(),
            "data-editing": editing.to_string(),
            onclick: move |_| {
                if editable && trigger == DataTableEditTrigger::Click {
                    active_cell.set(Some(editor_address.clone()));
                }
            },
            ondoubleclick: move |_| {
                if editable && trigger == DataTableEditTrigger::DoubleClick {
                    active_cell.set(Some(double_click_address.clone()));
                }
            },
            onkeydown: move |event: KeyboardEvent| match event.key() {
                Key::Enter | Key::F2 if editable => {
                    event.prevent_default();
                    active_cell.set(Some(keyboard_address.clone()));
                }
                Key::Escape if editing => {
                    event.prevent_default();
                    active_cell.set(None);
                }
                _ => {}
            },
            if editing {
                if let Some(render_editor) = props.render_editor {
                    {render_editor.call(DataTableEditContext {
                        cell: context,
                        close: close_editor,
                    })}
                }
            } else {
                {props.render_cell.call(context)}
            }
        }
    }
}

fn table_error(error: &str) -> Element {
    rsx! {
        div { class: TABLE_ERROR_CLASS, role: "alert", "表格配置错误：{error}" }
    }
}

fn render_header_content(
    header: &super::layout::DataTableHeaderCell,
    leaves: &[ResolvedLeafColumn],
    render_header: Option<Callback<DataTableHeaderContext, Element>>,
) -> Element {
    let Some(leaf_key) = header.leaf_key.as_deref() else {
        return rsx! { "{header.title}" };
    };
    let Some(column) = leaves
        .iter()
        .find(|leaf| leaf.column.key == leaf_key)
        .map(|leaf| leaf.column.clone())
    else {
        return rsx! { "{header.title}" };
    };
    match render_header {
        Some(render_header) => render_header.call(DataTableHeaderContext { column }),
        None => rsx! { "{header.title}" },
    }
}

fn classes(base: &str, extra: Option<&str>) -> String {
    match extra {
        Some(extra) => format!("{base} {extra}"),
        None => base.to_owned(),
    }
}

fn header_class(fixed: DataTableFixed, group: bool) -> String {
    let fixed_class = match fixed {
        DataTableFixed::None => None,
        DataTableFixed::Left => Some(TABLE_FIXED_LEFT_CLASS),
        DataTableFixed::Right => Some(TABLE_FIXED_RIGHT_CLASS),
    };
    let class = classes(TABLE_HEADER_CELL_CLASS, fixed_class);
    if group {
        format!("{class} {TABLE_HEADER_GROUP_CLASS}")
    } else {
        class
    }
}

fn header_style(header: &super::layout::DataTableHeaderCell) -> String {
    let mut style = format!(
        "--data-table-header-level:{};width:{}px;min-width:{}px;",
        header.level, header.width, header.width
    );
    push_fixed_offset(&mut style, header.fixed, header.offset);
    style
}

fn body_cell_class(leaf: &ResolvedLeafColumn, editable: bool, editing: bool) -> String {
    let fixed_class = match leaf.column.fixed {
        DataTableFixed::None => None,
        DataTableFixed::Left => Some(TABLE_FIXED_LEFT_CLASS),
        DataTableFixed::Right => Some(TABLE_FIXED_RIGHT_CLASS),
    };
    let mut class = classes(TABLE_CELL_CLASS, fixed_class);
    if editable {
        class.push(' ');
        class.push_str(TABLE_CELL_EDITABLE_CLASS);
    }
    if editing {
        class.push(' ');
        class.push_str(TABLE_CELL_EDITING_CLASS);
    }
    let align_class = match leaf.column.align {
        DataTableAlign::Start => TABLE_ALIGN_START_CLASS,
        DataTableAlign::Center => TABLE_ALIGN_CENTER_CLASS,
        DataTableAlign::End => TABLE_ALIGN_END_CLASS,
    };
    class.push(' ');
    class.push_str(align_class);
    class
}

fn body_cell_style(leaf: &ResolvedLeafColumn, layout: DataTableBodyCell, width: u32) -> String {
    let mut style = format!("width:{width}px;min-width:{width}px;");
    if layout.colspan == 1 {
        push_fixed_offset(&mut style, leaf.column.fixed, leaf.left.or(leaf.right));
    }
    style
}

fn push_fixed_offset(style: &mut String, fixed: DataTableFixed, offset: Option<u32>) {
    let Some(offset) = offset else {
        return;
    };
    match fixed {
        DataTableFixed::None => {}
        DataTableFixed::Left => style.push_str(&format!("left:{offset}px;")),
        DataTableFixed::Right => style.push_str(&format!("right:{offset}px;")),
    }
}
