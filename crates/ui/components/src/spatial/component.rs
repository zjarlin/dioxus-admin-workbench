use dioxus::prelude::*;

/// 图节点的交互状态。
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum GraphNodeState {
    #[default]
    Default,
    Dragging,
    ConnectionSource,
}

impl GraphNodeState {
    fn value(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Dragging => "dragging",
            Self::ConnectionSource => "connection-source",
        }
    }
}

/// 承载可定位节点的二维画布。
#[component]
pub fn GraphCanvas(
    width: i32,
    height: i32,
    onpointermove: Option<EventHandler<PointerEvent>>,
    onpointerup: Option<EventHandler<PointerEvent>>,
    onpointercancel: Option<EventHandler<PointerEvent>>,
    onpointerleave: Option<EventHandler<PointerEvent>>,
    children: Element,
) -> Element {
    let bounds = format!("width:{width}px;height:{height}px;");
    rsx! {
        div {
            class: "dx-graph-canvas",
            style: bounds,
            onpointermove: move |event| {
                if let Some(handler) = onpointermove {
                    handler.call(event);
                }
            },
            onpointerup: move |event| {
                if let Some(handler) = onpointerup {
                    handler.call(event);
                }
            },
            onpointercancel: move |event| {
                if let Some(handler) = onpointercancel {
                    handler.call(event);
                }
            },
            onpointerleave: move |event| {
                if let Some(handler) = onpointerleave {
                    handler.call(event);
                }
            },
            {children}
        }
    }
}

/// 在二维画布内定位单个节点。
#[component]
pub fn GraphNode(
    left: i32,
    top: i32,
    #[props(default)] state: GraphNodeState,
    aria_label: String,
    children: Element,
) -> Element {
    let position = format!("left:{left}px;top:{top}px;");
    rsx! {
        article {
            class: "dx-graph-node",
            "data-state": state.value(),
            style: position,
            aria_label,
            {children}
        }
    }
}

/// 根据树深度缩进单元格内容。
#[component]
pub fn TreeIndent(depth: usize, #[props(default)] root: bool, children: Element) -> Element {
    let indent = format!("--dx-tree-depth:{depth};");
    rsx! {
        div {
            class: "dx-tree-indent",
            "data-root": root,
            style: indent,
            {children}
        }
    }
}
