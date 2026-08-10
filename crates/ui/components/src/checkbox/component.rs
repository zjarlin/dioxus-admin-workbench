use std::{ops::Not, rc::Rc};

use crate::attributes::with_class;
use dioxus::prelude::*;
use dioxus_icons::lucide::Check;

#[css_module("/src/checkbox/style.css")]
struct Styles;

pub(crate) fn load_stylesheet() {
    drop(Styles::dx_checkbox.to_string());
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckboxState {
    Checked,
    Indeterminate,
    Unchecked,
}

impl CheckboxState {
    fn aria_checked(self) -> &'static str {
        match self {
            Self::Checked => "true",
            Self::Indeterminate => "mixed",
            Self::Unchecked => "false",
        }
    }

    fn data_state(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Indeterminate => "indeterminate",
            Self::Unchecked => "unchecked",
        }
    }
}

impl From<CheckboxState> for bool {
    fn from(value: CheckboxState) -> Self {
        !matches!(value, CheckboxState::Unchecked)
    }
}

impl Not for CheckboxState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Unchecked => Self::Checked,
            Self::Checked | Self::Indeterminate => Self::Unchecked,
        }
    }
}

#[derive(Clone, Copy)]
struct CheckboxContext {
    checked: Memo<CheckboxState>,
    disabled: ReadSignal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    #[props(default)]
    pub checked: ReadSignal<Option<CheckboxState>>,
    #[props(default = CheckboxState::Unchecked)]
    pub default_checked: CheckboxState,
    #[props(default)]
    pub required: ReadSignal<bool>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default)]
    pub name: ReadSignal<String>,
    #[props(default = ReadSignal::new(Signal::new(String::from("on"))))]
    pub value: ReadSignal<String>,
    #[props(default)]
    pub on_checked_change: Callback<CheckboxState>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let mut internal = use_signal(|| (props.checked)().unwrap_or(props.default_checked));
    let checked = use_memo(move || (props.checked)().unwrap_or_else(|| internal()));
    let mut button_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let attributes = with_class(props.attributes, Styles::dx_checkbox.to_string());

    use_context_provider(|| CheckboxContext {
        checked,
        disabled: props.disabled,
    });

    rsx! {
        button {
            r#type: "button",
            value: props.value,
            role: "checkbox",
            aria_checked: checked().aria_checked(),
            aria_required: props.required,
            disabled: props.disabled,
            "data-state": checked().data_state(),
            "data-disabled": props.disabled,
            onmounted: move |event| button_ref.set(Some(event.data())),
            onclick: move |_| {
                let next = !checked();
                if (props.checked)().is_none() {
                    internal.set(next);
                }
                props.on_checked_change.call(next);
                if let Some(node) = button_ref() {
                    spawn(async move {
                        let _ = node.set_focus(true).await;
                    });
                }
            },
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    event.prevent_default();
                }
            },
            ..attributes,
            CheckboxIndicator {}
        }
        input {
            r#type: "checkbox",
            name: props.name,
            value: props.value,
            required: props.required,
            disabled: props.disabled,
            checked: bool::from(checked()),
            aria_hidden: "true",
            tabindex: "-1",
            style: "position:absolute;pointer-events:none;opacity:0;margin:0;",
        }
    }
}

#[component]
fn CheckboxIndicator() -> Element {
    let context: CheckboxContext = use_context();
    let checked = (context.checked)();
    rsx! {
        span {
            class: Styles::dx_checkbox_indicator,
            "data-state": checked.data_state(),
            "data-disabled": context.disabled,
            if bool::from(checked) {
                Check { size: "1rem" }
            }
        }
    }
}
