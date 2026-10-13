use crate::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{CircleAlert, CircleCheck, LoaderCircle, RefreshCw};

#[component]
pub fn PageSurface(children: Element) -> Element {
    rsx! { section { class: "admin-page", {children} } }
}

#[component]
pub fn PageHeader(title: String, #[props(default)] detail: String, children: Element) -> Element {
    rsx! {
        header { class: "admin-page-header",
            div { class: "admin-page-heading",
                h1 { "{title}" }
                if !detail.is_empty() { p { "{detail}" } }
            }
            div { class: "admin-actions", {children} }
        }
    }
}

#[component]
pub fn StatusMessage(message: String, #[props(default)] error: bool) -> Element {
    rsx! {
        p { class: "admin-feedback", "data-error": error.to_string(), role: if error { "alert" } else { "status" },
            if error { CircleAlert {} } else { CircleCheck {} }
            span { "{message}" }
        }
    }
}

#[component]
pub fn EmptyState(title: String, #[props(default)] detail: String, children: Element) -> Element {
    rsx! { div { class: "admin-empty-state", role: "status",
        h2 { "{title}" }
        if !detail.is_empty() { p { "{detail}" } }
        {children}
    } }
}

#[component]
pub fn RequestState(
    #[props(default)] error: Option<String>,
    #[props(default)] on_retry: Option<Callback<()>>,
) -> Element {
    rsx! {
        div { class: "admin-request-state", role: if error.is_some() { "alert" } else { "status" },
            if let Some(ref message) = error {
                CircleAlert {}
                p { "{message}" }
                if let Some(retry) = on_retry {
                    Button { variant: ButtonVariant::Outline, onclick: move |_| retry.call(()), RefreshCw {} "重试" }
                }
            } else {
                div { class: "admin-request-state__pending",
                    LoaderCircle { class: "admin-request-state__spinner" }
                    p { "正在加载" }
                }
                // 骨架屏按页面结构（标题、工具条、若干行）错落排列并逐行错峰扫光，
                // 让短暂等待也能看出内容正在成形，而不是几段等长色块。
                div { class: "admin-skeleton-list", aria_hidden: "true",
                    div { class: "admin-skeleton admin-skeleton--title" }
                    div { class: "admin-skeleton-toolbar",
                        div { class: "admin-skeleton admin-skeleton--field" }
                        div { class: "admin-skeleton admin-skeleton--action" }
                    }
                    div { class: "admin-skeleton-rows",
                        for index in 0..5 {
                            div { class: "admin-skeleton-row", "data-row": index.to_string(),
                                for _ in 0..4 {
                                    div { class: "admin-skeleton admin-skeleton--cell" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
