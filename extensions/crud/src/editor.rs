use az_dioxus_admin_shell::PageExtensionEditorContext;
use az_ui_components::input::Input;
use dioxus::prelude::*;

use crate::CrudPageConfig;

#[component]
pub(crate) fn CrudEditor(context: PageExtensionEditorContext) -> Element {
    let config =
        serde_json::from_value::<CrudPageConfig>(context.config.clone()).unwrap_or_default();
    rsx! {
        fieldset {
            legend { "资源增删改查配置" }
            label { r#for: "crud-resource", "资源" }
            select {
                id: "crud-resource",
                name: "resource_id",
                value: "{config.resource_id}",
                onchange: {
                    let on_change = context.on_change;
                    let page_size = config.page_size;
                    move |event: FormEvent| {
                        let next = CrudPageConfig {
                            resource_id: event.value(),
                            page_size,
                        };
                        on_change.call(serde_json::json!({
                            "resource_id": next.resource_id,
                            "page_size": next.page_size,
                        }));
                    }
                },
                option { value: "", "选择资源" }
                for resource in context.resources.resources.values() {
                    option {
                        value: "{resource.id}",
                        selected: config.resource_id == resource.id,
                        "{resource.title} · {resource.name}"
                    }
                }
            }
            label { r#for: "crud-page-size", "每页条数" }
            Input {
                id: "crud-page-size",
                name: "page_size",
                r#type: "number",
                min: "1",
                max: "200",
                value: "{config.page_size}",
                oninput: {
                    let on_change = context.on_change;
                    let resource_id = config.resource_id.clone();
                    move |event: FormEvent| {
                        let page_size = event.value().parse::<u32>().unwrap_or(20);
                        let next = CrudPageConfig {
                            resource_id: resource_id.clone(),
                            page_size,
                        };
                        on_change.call(serde_json::json!({
                            "resource_id": next.resource_id,
                            "page_size": next.page_size,
                        }));
                    }
                },
            }
        }
    }
}
