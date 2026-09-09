use dioxus::prelude::*;

const INPUT_CLASS: &str = "dx-input";

/// 带可见标签和双向值回调的常用文本输入框。
#[component]
pub fn TextInput(
    label: String,
    value: String,
    on_change: EventHandler<String>,
    #[props(default)] input_type: String,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] autocomplete: Option<String>,
) -> Element {
    let id = format!("input-{}", label.to_lowercase().replace(' ', "-"));
    rsx! {
        fieldset { class: "grid gap-2",
            label { class: "text-sm font-medium", r#for: "{id}", "{label}" }
            Input {
                id: id.clone(),
                r#type: input_type,
                placeholder: placeholder,
                autocomplete: autocomplete,
                aria_label: label,
                value: value,
                oninput: move |event: FormEvent| on_change.call(event.value()),
            }
        }
    }
}

#[component]
pub fn Input(
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
    oninvalid: Option<EventHandler<FormEvent>>,
    onselect: Option<EventHandler<SelectionEvent>>,
    onselectionchange: Option<EventHandler<SelectionEvent>>,
    onfocus: Option<EventHandler<FocusEvent>>,
    onblur: Option<EventHandler<FocusEvent>>,
    onfocusin: Option<EventHandler<FocusEvent>>,
    onfocusout: Option<EventHandler<FocusEvent>>,
    onmounted: Option<EventHandler<MountedEvent>>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    onkeypress: Option<EventHandler<KeyboardEvent>>,
    onkeyup: Option<EventHandler<KeyboardEvent>>,
    onwheel: Option<EventHandler<WheelEvent>>,
    oncompositionstart: Option<EventHandler<CompositionEvent>>,
    oncompositionupdate: Option<EventHandler<CompositionEvent>>,
    oncompositionend: Option<EventHandler<CompositionEvent>>,
    oncopy: Option<EventHandler<ClipboardEvent>>,
    oncut: Option<EventHandler<ClipboardEvent>>,
    onpaste: Option<EventHandler<ClipboardEvent>>,
    #[props(extends=GlobalAttributes)]
    #[props(extends=input)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        input {
            class: INPUT_CLASS,
            oninput: move |e| _ = oninput.map(|callback| callback(e)),
            onchange: move |e| _ = onchange.map(|callback| callback(e)),
            oninvalid: move |e| _ = oninvalid.map(|callback| callback(e)),
            onselect: move |e| _ = onselect.map(|callback| callback(e)),
            onselectionchange: move |e| _ = onselectionchange.map(|callback| callback(e)),
            onfocus: move |e| _ = onfocus.map(|callback| callback(e)),
            onblur: move |e| _ = onblur.map(|callback| callback(e)),
            onfocusin: move |e| _ = onfocusin.map(|callback| callback(e)),
            onfocusout: move |e| _ = onfocusout.map(|callback| callback(e)),
            onmounted: move |e| _ = onmounted.map(|callback| callback(e)),
            onkeydown: move |e| _ = onkeydown.map(|callback| callback(e)),
            onkeypress: move |e| _ = onkeypress.map(|callback| callback(e)),
            onkeyup: move |e| _ = onkeyup.map(|callback| callback(e)),
            onwheel: move |e| _ = onwheel.map(|callback| callback(e)),
            oncompositionstart: move |e| _ = oncompositionstart.map(|callback| callback(e)),
            oncompositionupdate: move |e| _ = oncompositionupdate.map(|callback| callback(e)),
            oncompositionend: move |e| _ = oncompositionend.map(|callback| callback(e)),
            oncopy: move |e| _ = oncopy.map(|callback| callback(e)),
            oncut: move |e| _ = oncut.map(|callback| callback(e)),
            onpaste: move |e| _ = onpaste.map(|callback| callback(e)),
            ..attributes,
            {children}
        }
    }
}
