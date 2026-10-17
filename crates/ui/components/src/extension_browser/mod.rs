use dioxus::prelude::*;

#[component]
pub fn ExtensionBrowser(
    sidebar: Element,
    children: Element,
    detail_open: bool,
    #[props(default)] sidebar_scroll_id: Option<String>,
    #[props(default)] detail_scroll_id: Option<String>,
) -> Element {
    rsx! {
        div { class: "extension-browser", "data-detail-open": detail_open,
            aside { class: "extension-browser__sidebar", "data-url-scroll": sidebar_scroll_id, aria_label: "插件列表", {sidebar} }
            div { class: "extension-browser__separator", role: "separator", tabindex: 0, "aria-orientation": "vertical", "aria-label": "调整插件列表宽度",
                onmounted: move |_| { spawn(async { let _ = document::eval(include_str!("resize.js")).await; }); },
            }
            section { class: "extension-browser__detail", "data-url-scroll": detail_scroll_id, aria_label: "插件详情", {children} }
        }
    }
}
