use dioxus::prelude::*;

#[component]
pub fn ExtensionBrowser(sidebar: Element, children: Element, detail_open: bool) -> Element {
    rsx! {
        div { class: "extension-browser", "data-detail-open": detail_open,
            aside { class: "extension-browser__sidebar", aria_label: "插件列表", {sidebar} }
            div { class: "extension-browser__separator", role: "separator", tabindex: 0, "aria-orientation": "vertical", "aria-label": "调整插件列表宽度",
                onmounted: move |_| { spawn(async { let _ = document::eval(include_str!("resize.js")).await; }); },
            }
            section { class: "extension-browser__detail", aria_label: "插件详情", {children} }
        }
    }
}
