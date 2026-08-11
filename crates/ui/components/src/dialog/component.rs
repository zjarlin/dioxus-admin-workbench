use std::sync::atomic::{AtomicUsize, Ordering};

use crate::attributes::with_class;
use dioxus::prelude::*;

const DIALOG_CLASS: &str = "dx-dialog";
const DIALOG_BACKDROP_CLASS: &str = "dx-dialog-backdrop";
const DIALOG_TITLE_CLASS: &str = "dx-dialog-title";
const DIALOG_DESCRIPTION_CLASS: &str = "dx-dialog-description";

fn unique_id(prefix: &str) -> String {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{id}")
}

#[derive(Clone, Copy)]
struct DialogContext {
    title_id: Signal<String>,
    description_id: Signal<String>,
}

#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    #[props(default)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Callback<bool>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let mut internal = use_signal(|| (props.open)().unwrap_or(props.default_open));
    let open = use_memo(move || (props.open)().unwrap_or_else(|| internal()));
    let root_id = use_signal(|| (props.id)().unwrap_or_else(|| unique_id("dialog")));
    let title_id = use_signal(|| unique_id("dialog-title"));
    let description_id = use_signal(|| unique_id("dialog-description"));
    let attributes = with_class(props.attributes, DIALOG_CLASS.to_owned());

    use_context_provider(|| DialogContext {
        title_id,
        description_id,
    });

    let close = Callback::new(move |_: ()| {
        if (props.open)().is_none() {
            internal.set(false);
        }
        props.on_open_change.call(false);
    });

    if !open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: DIALOG_BACKDROP_CLASS,
            "data-state": "open",
            onclick: move |_| close.call(()),
            div {
                id: root_id,
                role: "dialog",
                aria_modal: props.is_modal,
                aria_labelledby: title_id,
                aria_describedby: description_id,
                tabindex: "-1",
                onmounted: move |event| {
                    let node = event.data();
                    spawn(async move {
                        let _ = node.set_focus(true).await;
                    });
                },
                onkeydown: move |event| {
                    if event.key() == Key::Escape {
                        event.prevent_default();
                        close.call(());
                    }
                },
                onclick: move |event| event.stop_propagation(),
                ..attributes,
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DialogTitleProps {
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    let context: DialogContext = use_context();
    let id = (props.id)().unwrap_or_else(|| (context.title_id)());
    let attributes = with_class(props.attributes, DIALOG_TITLE_CLASS.to_owned());
    rsx! {
        h2 { id, ..attributes, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DialogDescriptionProps {
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    let context: DialogContext = use_context();
    let id = (props.id)().unwrap_or_else(|| (context.description_id)());
    let attributes = with_class(props.attributes, DIALOG_DESCRIPTION_CLASS.to_owned());
    rsx! {
        p { id, ..attributes, {props.children} }
    }
}
