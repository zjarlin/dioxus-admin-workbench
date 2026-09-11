use crate::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{CircleAlert, CircleCheck, RefreshCw};

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
                p { "正在加载" }
                for _ in 0..4 { div { class: "admin-skeleton", aria_hidden: "true" } }
            }
        }
    }
}
