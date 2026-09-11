use super::{AsyncResult, StatusMessage};
use crate::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::Trash2;

#[component]
pub fn DeleteRecordsDialog<R: Clone + PartialEq + 'static>(
    title: String,
    items: Vec<R>,
    item_label: Callback<R, String>,
    delete: Callback<R, AsyncResult<()>>,
    on_close: Callback<()>,
    on_deleted: Callback<usize>,
    #[props(default = "所选记录将永久删除，无法撤销。".to_owned())] warning: String,
) -> Element {
    let mut pending = use_signal(|| items);
    let mut busy = use_signal(|| false);
    let mut completed = use_signal(|| 0_usize);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        Dialog { open: true, on_open_change: move |open: bool| if !open && !busy() { on_close.call(()) },
            DialogTitle { "{title}" }
            DialogDescription { "{warning}" }
            ul { class: "admin-delete-targets", for item in pending() { li { "{item_label.call(item.clone())}" } } }
            if busy() { p { role: "status", "正在处理 {completed()} / {pending().len()} 项" } }
            if let Some(message) = error() { StatusMessage { error: true, message } }
            footer { class: "admin-form-footer",
                Button { variant: ButtonVariant::Outline, disabled: busy(), onclick: move |_| on_close.call(()), "取消" }
                Button { variant: ButtonVariant::Destructive, disabled: busy() || pending().is_empty(), onclick: move |_| {
                    if busy() { return; }
                    busy.set(true); error.set(None); completed.set(0);
                    spawn(async move {
                        let targets = pending();
                        let total = targets.len();
                        let mut failed = Vec::new();
                        let mut messages = Vec::new();
                        for item in targets {
                            if let Err(message) = delete.call(item.clone()).await {
                                messages.push(format!("{}：{message}", item_label.call(item.clone())));
                                failed.push(item);
                            }
                            completed += 1;
                        }
                        busy.set(false);
                        if total > failed.len() { on_deleted.call(total - failed.len()); }
                        if failed.is_empty() { on_close.call(()); }
                        else { pending.set(failed); error.set(Some(messages.join("；"))); }
                    });
                }, Trash2 {} if busy() { "正在删除" } else { "确认删除" } }
            }
        }
    }
}
