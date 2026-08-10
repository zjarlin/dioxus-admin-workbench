use az_admin_shell_core::{
    ResourceDefinition, ResourceFieldDefinition, ResourceFieldKind, ResourcePage, ResourceRecord,
    ResourceRequest, ResourceResponse,
};
use az_dioxus_admin_shell::{AdminProviderHandle, PageExtensionRuntimeContext};
use az_ui_components::{
    button::{Button, ButtonSize, ButtonVariant},
    data_table::{DataTable, DataTableCellContext, DataTableColumn, DataTableFixed},
    dialog::{Dialog, DialogDescription, DialogTitle},
    input::Input,
};
use dioxus::prelude::*;
use icons::{ChevronLeft, ChevronRight, Pencil, Plus, Trash2};
use serde_json::Value;

use crate::CrudPageConfig;

#[derive(Clone, Debug, PartialEq)]
enum RecordDialog {
    Create,
    Edit(ResourceRecord),
    Delete(ResourceRecord),
}

pub(crate) fn render(context: PageExtensionRuntimeContext) -> Element {
    let config = match serde_json::from_value::<CrudPageConfig>(context.payload) {
        Ok(config) => config,
        Err(error) => return error_state(&format!("CRUD 配置无效: {error}")),
    };
    let Some(resource) = context.resources.get(&config.resource_id).cloned() else {
        return error_state(&format!("CRUD 资源不存在: {}", config.resource_id));
    };
    rsx! {
        CrudPage {
            title: context.page.title,
            config,
            resource,
            admin: context.admin,
        }
    }
}

#[component]
fn CrudPage(
    title: String,
    config: CrudPageConfig,
    resource: ResourceDefinition,
    admin: AdminProviderHandle,
) -> Element {
    let generation = use_signal(|| 0_u64);
    let mut page_number = use_signal(|| 0_u32);
    let mut dialog = use_signal(|| None::<RecordDialog>);
    let mut notice = use_signal(|| None::<String>);
    let page_size = config.page_size;
    let resource_id = resource.id.clone();
    let provider = admin.provider().clone();
    let records = use_resource(move || {
        let _generation = generation();
        let request = ResourceRequest::List {
            resource_id: resource_id.clone(),
            page: page_number(),
            page_size,
        };
        let provider = provider.clone();
        async move {
            match provider.execute_resource(request).await {
                Ok(ResourceResponse::Page(page)) => Ok(page),
                Ok(_) => Err("资源列表返回了错误的结果类型".to_owned()),
                Err(error) => Err(error.to_string()),
            }
        }
    });
    let result = records.read().as_ref().cloned();
    let page = result
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or(ResourcePage {
            items: Vec::new(),
            total: 0,
        });
    let columns = table_columns(&resource);
    let empty_text = match result.as_ref() {
        None => "正在加载".to_owned(),
        Some(Ok(_)) => "暂无数据".to_owned(),
        Some(Err(error)) => error.clone(),
    };
    let has_previous = page_number() > 0;
    let has_next = u64::from(page_number().saturating_add(1)) * u64::from(page_size) < page.total;

    rsx! {
        section {
            header {
                div {
                    h2 { "{title}" }
                    p { "{resource.title}" }
                }
                if resource.operations.create {
                    Button { onclick: move |_| dialog.set(Some(RecordDialog::Create)),
                        Plus { class: "size-4" }
                        "新增"
                    }
                }
            }
            if let Some(message) = notice() {
                div { role: "status", "{message}" }
            }
            DataTable::<ResourceRecord> {
                aria_label: format!("{}数据表", resource.title),
                rows: page.items.clone(),
                columns,
                max_height: "calc(100dvh - 13rem)",
                empty_text,
                row_key: {
                    let id_field = resource.id_field.clone();
                    move |record: ResourceRecord| record_id(&record, &id_field)
                },
                render_cell: {
                    let resource = resource.clone();
                    move |cell: DataTableCellContext<ResourceRecord>| {
                        render_cell(cell, &resource, dialog)
                    }
                },
            }
            footer {
                span { "共 {page.total} 条" }
                Button {
                    size: ButtonSize::IconSm,
                    variant: ButtonVariant::Outline,
                    disabled: !has_previous,
                    aria_label: "上一页",
                    onclick: move |_| page_number.set(page_number().saturating_sub(1)),
                    ChevronLeft { class: "size-4" }
                }
                span { "第 {page_number() + 1} 页" }
                Button {
                    size: ButtonSize::IconSm,
                    variant: ButtonVariant::Outline,
                    disabled: !has_next,
                    aria_label: "下一页",
                    onclick: move |_| page_number.set(page_number().saturating_add(1)),
                    ChevronRight { class: "size-4" }
                }
            }
            if let Some(value) = dialog() {
                RecordDialogView {
                    value,
                    resource: resource.clone(),
                    admin: admin.clone(),
                    dialog,
                    generation,
                    on_status: Callback::new(move |message| notice.set(Some(message))),
                }
            }
        }
    }
}

fn table_columns(resource: &ResourceDefinition) -> Vec<DataTableColumn> {
    let mut columns = resource
        .fields
        .iter()
        .filter(|field| field.list_visible)
        .map(|field| DataTableColumn::leaf(field.name.clone(), field.title.clone()).width(180))
        .collect::<Vec<_>>();
    if resource.operations.update || resource.operations.delete {
        columns.push(
            DataTableColumn::leaf("__actions", "操作")
                .width(120)
                .fixed(DataTableFixed::Right),
        );
    }
    columns
}

fn render_cell(
    cell: DataTableCellContext<ResourceRecord>,
    resource: &ResourceDefinition,
    mut dialog: Signal<Option<RecordDialog>>,
) -> Element {
    if cell.column.key == "__actions" {
        let edit_record = cell.row.clone();
        let delete_record = cell.row;
        return rsx! {
            if resource.operations.update {
                Button {
                    size: ButtonSize::IconSm,
                    variant: ButtonVariant::Ghost,
                    title: "编辑",
                    aria_label: "编辑",
                    onclick: move |event: MouseEvent| {
                        event.stop_propagation();
                        dialog.set(Some(RecordDialog::Edit(edit_record.clone())));
                    },
                    Pencil { class: "size-4" }
                }
            }
            if resource.operations.delete {
                Button {
                    size: ButtonSize::IconSm,
                    variant: ButtonVariant::Ghost,
                    title: "删除",
                    aria_label: "删除",
                    onclick: move |event: MouseEvent| {
                        event.stop_propagation();
                        dialog.set(Some(RecordDialog::Delete(delete_record.clone())));
                    },
                    Trash2 { class: "size-4" }
                }
            }
        };
    }
    let value = cell.row.get(&cell.column.key).unwrap_or(&Value::Null);
    rsx! { "{display_value(value)}" }
}

#[component]
fn RecordDialogView(
    value: RecordDialog,
    resource: ResourceDefinition,
    admin: AdminProviderHandle,
    mut dialog: Signal<Option<RecordDialog>>,
    mut generation: Signal<u64>,
    on_status: Callback<String>,
) -> Element {
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let existing = match &value {
        RecordDialog::Edit(record) | RecordDialog::Delete(record) => Some(record.clone()),
        RecordDialog::Create => None,
    };
    let deleting = matches!(value, RecordDialog::Delete(_));
    let title = match value {
        RecordDialog::Create => format!("新增{}", resource.title),
        RecordDialog::Edit(_) => format!("编辑{}", resource.title),
        RecordDialog::Delete(_) => format!("删除{}", resource.title),
    };

    rsx! {
        Dialog {
            open: true,
            on_open_change: move |open: bool| {
                if !open {
                    dialog.set(None);
                }
            },
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let request = if deleting {
                        let Some(record) = existing.as_ref() else {
                            error.set(Some("删除记录不存在".to_owned()));
                            return;
                        };
                        ResourceRequest::Delete {
                            resource_id: resource.id.clone(),
                            record_id: record_id(record, &resource.id_field),
                        }
                    } else {
                        let values = match form_values(&event, &resource.fields) {
                            Ok(values) => values,
                            Err(message) => {
                                error.set(Some(message));
                                return;
                            }
                        };
                        match existing.as_ref() {
                            Some(record) => ResourceRequest::Update {
                                resource_id: resource.id.clone(),
                                record_id: record_id(record, &resource.id_field),
                                values,
                            },
                            None => ResourceRequest::Create {
                                resource_id: resource.id.clone(),
                                values,
                            },
                        }
                    };
                    let provider = admin.provider().clone();
                    let success = if deleting { "已删除记录" } else { "已保存记录" }.to_owned();
                    pending.set(true);
                    spawn(async move {
                        match provider.execute_resource(request).await {
                            Ok(_) => {
                                generation.with_mut(|value| *value = value.saturating_add(1));
                                on_status.call(success);
                                dialog.set(None);
                            }
                            Err(cause) => error.set(Some(cause.to_string())),
                        }
                        pending.set(false);
                    });
                },
                DialogTitle { "{title}" }
                DialogDescription {
                    if deleting {
                        "确认删除该记录？此操作不可恢复。"
                    } else {
                        "根据资源字段契约保存正式数据。"
                    }
                }
                if !deleting {
                    for field in resource.fields.iter().filter(|field| field.form_visible) {
                        {field_editor(field, existing.as_ref())}
                    }
                }
                if let Some(message) = error() {
                    p { role: "alert", "{message}" }
                }
                footer {
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| dialog.set(None),
                        "取消"
                    }
                    Button {
                        r#type: "submit",
                        disabled: pending(),
                        variant: if deleting {
                            ButtonVariant::Destructive
                        } else {
                            ButtonVariant::Primary
                        },
                        if deleting { "确认删除" } else { "保存" }
                    }
                }
            }
        }
    }
}

fn field_editor(field: &ResourceFieldDefinition, record: Option<&ResourceRecord>) -> Element {
    let value = record
        .and_then(|record| record.get(&field.name))
        .map(display_value)
        .unwrap_or_default();
    if field.kind == ResourceFieldKind::Boolean {
        return rsx! {
            label { r#for: "field-{field.name}", "{field.title}" }
            select {
                id: "field-{field.name}",
                name: "{field.name}",
                required: field.required,
                option { value: "false", selected: value == "false", "否" }
                option { value: "true", selected: value == "true", "是" }
            }
        };
    }
    rsx! {
        label { r#for: "field-{field.name}", "{field.title}" }
        Input {
            id: "field-{field.name}",
            name: "{field.name}",
            required: field.required,
            value,
            r#type: match field.kind {
                ResourceFieldKind::Integer | ResourceFieldKind::Decimal => "number",
                ResourceFieldKind::Timestamp => "datetime-local",
                ResourceFieldKind::Text | ResourceFieldKind::Boolean | ResourceFieldKind::Json => "text",
            },
        }
    }
}

fn form_values(
    event: &FormEvent,
    fields: &[ResourceFieldDefinition],
) -> Result<ResourceRecord, String> {
    fields
        .iter()
        .filter(|field| field.form_visible)
        .map(|field| {
            let value = form_text(event, &field.name);
            parse_value(field, &value).map(|value| (field.name.clone(), value))
        })
        .collect()
}

fn parse_value(field: &ResourceFieldDefinition, value: &str) -> Result<Value, String> {
    if value.trim().is_empty() && !field.required {
        return Ok(Value::Null);
    }
    match field.kind {
        ResourceFieldKind::Text | ResourceFieldKind::Timestamp => {
            Ok(Value::String(value.to_owned()))
        }
        ResourceFieldKind::Integer => value
            .parse::<i64>()
            .map(Value::from)
            .map_err(|error| format!("{}不是有效整数: {error}", field.title)),
        ResourceFieldKind::Decimal => value
            .parse::<f64>()
            .map(Value::from)
            .map_err(|error| format!("{}不是有效数字: {error}", field.title)),
        ResourceFieldKind::Boolean => value
            .parse::<bool>()
            .map(Value::from)
            .map_err(|error| format!("{}不是有效布尔值: {error}", field.title)),
        ResourceFieldKind::Json => serde_json::from_str(value)
            .map_err(|error| format!("{}不是有效 JSON: {error}", field.title)),
    }
}

fn form_text(event: &FormEvent, name: &str) -> String {
    match event.get_first(name) {
        Some(dioxus::html::FormValue::Text(value)) => value,
        _ => String::new(),
    }
}

fn record_id(record: &ResourceRecord, id_field: &str) -> String {
    record
        .get(id_field)
        .map(display_value)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| serde_json::to_string(record).unwrap_or_default())
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn error_state(message: &str) -> Element {
    rsx! { div { role: "alert", "{message}" } }
}
