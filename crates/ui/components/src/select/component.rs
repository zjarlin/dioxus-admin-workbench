use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::attributes::with_class;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, ChevronDown};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectItem {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectPlacement {
    Top,
    #[default]
    Bottom,
}

impl SelectPlacement {
    fn data_value(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    pub value: ReadSignal<String>,
    pub options: Vec<SelectItem>,
    #[props(default)]
    pub name: ReadSignal<String>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(String::from("请选择"))))]
    pub placeholder: ReadSignal<String>,
    #[props(default)]
    pub aria_label: ReadSignal<String>,
    #[props(default)]
    pub placement: SelectPlacement,
    #[props(default)]
    pub on_value_change: Callback<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    let mut open = use_signal(|| false);
    let mut active_index = use_signal(|| first_enabled_index(&props.options));
    let mut trigger_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let mut list_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let root_id = use_signal(|| unique_id("select"));
    let attributes = with_class(props.attributes, "dx-select".to_owned());
    let selected_label = props
        .options
        .iter()
        .find(|option| option.value == (props.value)())
        .map(|option| option.label.clone());
    let display_value = selected_label.unwrap_or_else(|| (props.placeholder)());
    let placeholder = !props
        .options
        .iter()
        .any(|option| option.value == (props.value)());

    use_effect(move || {
        if !open() {
            return;
        }
        let Some(list_ref) = list_ref() else {
            return;
        };
        spawn(async move {
            let _ = list_ref.set_focus(true).await;
        });
    });

    let options_for_trigger = props.options.clone();
    let options_for_keyboard = props.options.clone();
    rsx! {
        div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": props.disabled,
            "data-placement": props.placement.data_value(),
            ..attributes,
            button {
                class: "dx-select-trigger",
                r#type: "button",
                disabled: props.disabled,
                aria_label: props.aria_label,
                aria_haspopup: "listbox",
                aria_expanded: open(),
                aria_controls: "{root_id}-list",
                onmounted: move |event| trigger_ref.set(Some(event.data())),
                onclick: move |_| {
                    let next = !open();
                    if next {
                        active_index.set(selected_or_first_enabled_index(
                            &options_for_trigger,
                            &(props.value)(),
                        ));
                    }
                    open.set(next);
                },
                onkeydown: move |event| {
                    match event.key() {
                        Key::ArrowDown => {
                            active_index.set(first_enabled_index(&options_for_keyboard));
                            open.set(true);
                            event.prevent_default();
                            event.stop_propagation();
                        }
                        Key::ArrowUp => {
                            active_index.set(last_enabled_index(&options_for_keyboard));
                            open.set(true);
                            event.prevent_default();
                            event.stop_propagation();
                        }
                        _ => {}
                    }
                },
                span { "data-placeholder": placeholder, "{display_value}" }
                ChevronDown { class: "dx-select-expand-icon", size: "16px" }
            }
            if open() {
                div {
                    class: "dx-select-dismiss",
                    aria_hidden: "true",
                    onclick: move |event| {
                        event.prevent_default();
                        event.stop_propagation();
                        open.set(false);
                    },
                }
                div {
                    id: "{root_id}-list",
                    class: "dx-select-list",
                    role: "listbox",
                    tabindex: "0",
                    aria_label: props.aria_label,
                    aria_activedescendant: active_index().map(|index| format!("{}-option-{index}", root_id())),
                    "data-state": "open",
                    onmounted: move |event| list_ref.set(Some(event.data())),
                    onblur: move |_| open.set(false),
                    onkeydown: move |event| {
                        match event.key() {
                            Key::ArrowDown => {
                                active_index.set(next_enabled_index(&props.options, active_index(), 1));
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::ArrowUp => {
                                active_index.set(next_enabled_index(&props.options, active_index(), -1));
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::Home => {
                                active_index.set(first_enabled_index(&props.options));
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::End => {
                                active_index.set(last_enabled_index(&props.options));
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::Enter => {
                                if let Some(index) = active_index() {
                                    if let Some(option) = props.options.get(index).filter(|option| !option.disabled) {
                                        props.on_value_change.call(option.value.clone());
                                        open.set(false);
                                        focus_trigger(trigger_ref);
                                    }
                                }
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::Character(value) if value == " " => {
                                if let Some(index) = active_index() {
                                    if let Some(option) = props.options.get(index).filter(|option| !option.disabled) {
                                        props.on_value_change.call(option.value.clone());
                                        open.set(false);
                                        focus_trigger(trigger_ref);
                                    }
                                }
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::Escape => {
                                open.set(false);
                                focus_trigger(trigger_ref);
                                event.prevent_default();
                                event.stop_propagation();
                            }
                            Key::Character(value) => {
                                if let Some(index) = matching_index(&props.options, &value) {
                                    active_index.set(Some(index));
                                }
                            }
                            _ => {}
                        }
                    },
                    for (index, option) in props.options.iter().enumerate() {
                        {
                            let option_value = option.value.clone();
                            let option_disabled = option.disabled;
                            rsx! {
                                div {
                                    id: "{root_id}-option-{index}",
                                    class: "dx-select-option",
                                    role: "option",
                                    aria_selected: (props.value)() == option.value,
                                    aria_disabled: option.disabled,
                                    "data-active": active_index() == Some(index),
                                    "data-disabled": option.disabled,
                                    onmousemove: move |_| {
                                        if !option_disabled {
                                            active_index.set(Some(index));
                                        }
                                    },
                                    onclick: move |event| {
                                        event.prevent_default();
                                        event.stop_propagation();
                                        if option_disabled {
                                            return;
                                        }
                                        props.on_value_change.call(option_value.clone());
                                        open.set(false);
                                        focus_trigger(trigger_ref);
                                    },
                                    span { "{option.label}" }
                                    if (props.value)() == option.value {
                                        Check { class: "dx-select-check-icon", size: "1rem" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !(props.name)().is_empty() {
                input {
                    r#type: "hidden",
                    name: props.name,
                    value: props.value,
                    disabled: props.disabled,
                    aria_hidden: "true",
                }
            }
        }
    }
}

fn unique_id(prefix: &str) -> String {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{id}")
}

fn focus_trigger(trigger_ref: Signal<Option<Rc<MountedData>>>) {
    let Some(trigger_ref) = trigger_ref() else {
        return;
    };
    spawn(async move {
        let _ = trigger_ref.set_focus(true).await;
    });
}

fn selected_or_first_enabled_index(options: &[SelectItem], selected: &str) -> Option<usize> {
    options
        .iter()
        .position(|option| !option.disabled && option.value == selected)
        .or_else(|| first_enabled_index(options))
}

fn first_enabled_index(options: &[SelectItem]) -> Option<usize> {
    options.iter().position(|option| !option.disabled)
}

fn last_enabled_index(options: &[SelectItem]) -> Option<usize> {
    options.iter().rposition(|option| !option.disabled)
}

fn next_enabled_index(
    options: &[SelectItem],
    current: Option<usize>,
    direction: isize,
) -> Option<usize> {
    if options.is_empty() {
        return None;
    }
    let start = current.unwrap_or_else(|| if direction > 0 { options.len() - 1 } else { 0 });
    for offset in 1..=options.len() {
        let index = (start as isize + direction * offset as isize)
            .rem_euclid(options.len() as isize) as usize;
        if !options[index].disabled {
            return Some(index);
        }
    }
    None
}

fn matching_index(options: &[SelectItem], search: &str) -> Option<usize> {
    let search = search.to_lowercase();
    options.iter().position(|option| {
        !option.disabled && option.label.to_lowercase().starts_with(search.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<SelectItem> {
        vec![
            SelectItem::new("first", "第一个").disabled(true),
            SelectItem::new("second", "第二个"),
            SelectItem::new("third", "第三个"),
        ]
    }

    #[test]
    fn keyboard_navigation_skips_disabled_options_and_loops() {
        let options = options();
        assert_eq!(first_enabled_index(&options), Some(1));
        assert_eq!(next_enabled_index(&options, Some(1), 1), Some(2));
        assert_eq!(next_enabled_index(&options, Some(2), 1), Some(1));
        assert_eq!(next_enabled_index(&options, Some(1), -1), Some(2));
    }

    #[test]
    fn text_search_ignores_disabled_options() {
        let options = options();
        assert_eq!(matching_index(&options, "第"), Some(1));
        assert_eq!(matching_index(&options, "第三"), Some(2));
    }
}
