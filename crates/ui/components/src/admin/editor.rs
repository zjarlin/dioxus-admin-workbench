use super::{AsyncResult, StatusMessage};
use crate::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
};
use dioxus::prelude::*;

#[component]
pub fn EditorDialog(
    title: String,
    description: String,
    save: Callback<(), AsyncResult<()>>,
    on_saved: Callback<()>,
    on_close: Callback<()>,
    children: Element,
    #[props(default = "保存".to_owned())] submit_label: String,
    #[props(default = "正在保存".to_owned())] pending_label: String,
) -> Element {
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        Dialog { open: true, on_open_change: move |open: bool| if !open && !busy() { on_close.call(()) },
            DialogTitle { "{title}" }
            DialogDescription { "{description}" }
            form { class: "admin-form", onsubmit: move |event: FormEvent| {
                event.prevent_default();
                if busy() { return; }
                busy.set(true); error.set(None);
                let request = save.call(());
                spawn(async move {
                    let result = request.await;
                    busy.set(false);
                    match result { Ok(()) => on_saved.call(()), Err(message) => error.set(Some(message)) }
                });
            },
                fieldset { class: "admin-form", disabled: busy(), {children} }
                if let Some(message) = error() { StatusMessage { error: true, message } }
                footer { class: "admin-form-footer",
                    Button { r#type: "button", variant: ButtonVariant::Outline, disabled: busy(), onclick: move |_| on_close.call(()), "取消" }
                    Button { r#type: "submit", disabled: busy(), if busy() { "{pending_label}" } else { "{submit_label}" } }
                }
            }
        }
    }
}
