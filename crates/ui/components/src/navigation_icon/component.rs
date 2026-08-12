use crate::attributes::with_class;
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    Bell, BookOpen, Bot, Boxes, Building2, CircleDashed, ClipboardList, Cloud, Cog, Contact, Cpu,
    Database, FileText, Folder, Gauge, HardDrive, House, KeyRound, Landmark, Layers, MapPinned,
    Menu, MessageSquare, MonitorCog, Network, Package, PanelTop, Radio, Server, Settings, Share2,
    Shield, SlidersHorizontal, Terminal, UserRound, UsersRound, Waypoints, Workflow,
};

pub const DEFAULT_NAVIGATION_ICON: &str = "panel_top";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavigationIconOption {
    pub value: &'static str,
    pub label: &'static str,
}

pub const NAVIGATION_ICON_OPTIONS: &[NavigationIconOption] = &[
    NavigationIconOption {
        value: "house",
        label: "首页",
    },
    NavigationIconOption {
        value: "panel_top",
        label: "页面",
    },
    NavigationIconOption {
        value: "user_round",
        label: "用户",
    },
    NavigationIconOption {
        value: "users_round",
        label: "用户组",
    },
    NavigationIconOption {
        value: "shield",
        label: "权限",
    },
    NavigationIconOption {
        value: "building_2",
        label: "组织",
    },
    NavigationIconOption {
        value: "book_open",
        label: "字典",
    },
    NavigationIconOption {
        value: "menu",
        label: "菜单",
    },
    NavigationIconOption {
        value: "clipboard_list",
        label: "审计",
    },
    NavigationIconOption {
        value: "key_round",
        label: "认证",
    },
    NavigationIconOption {
        value: "landmark",
        label: "租户",
    },
    NavigationIconOption {
        value: "message_square",
        label: "消息",
    },
    NavigationIconOption {
        value: "share_2",
        label: "社交",
    },
    NavigationIconOption {
        value: "map_pinned",
        label: "地区",
    },
    NavigationIconOption {
        value: "settings",
        label: "设置",
    },
    NavigationIconOption {
        value: "package",
        label: "资产",
    },
    NavigationIconOption {
        value: "folder",
        label: "文件",
    },
    NavigationIconOption {
        value: "database",
        label: "数据",
    },
    NavigationIconOption {
        value: "gauge",
        label: "概览",
    },
    NavigationIconOption {
        value: "server",
        label: "服务器",
    },
    NavigationIconOption {
        value: "hard_drive",
        label: "存储",
    },
    NavigationIconOption {
        value: "cpu",
        label: "设备",
    },
    NavigationIconOption {
        value: "radio",
        label: "遥测",
    },
    NavigationIconOption {
        value: "terminal",
        label: "终端",
    },
    NavigationIconOption {
        value: "bot",
        label: "智能体",
    },
    NavigationIconOption {
        value: "cloud",
        label: "云服务",
    },
    NavigationIconOption {
        value: "workflow",
        label: "流程",
    },
    NavigationIconOption {
        value: "network",
        label: "网络",
    },
    NavigationIconOption {
        value: "layers",
        label: "分层",
    },
    NavigationIconOption {
        value: "boxes",
        label: "模块",
    },
    NavigationIconOption {
        value: "file_text",
        label: "文档",
    },
    NavigationIconOption {
        value: "bell",
        label: "通知",
    },
    NavigationIconOption {
        value: "contact",
        label: "联系人",
    },
    NavigationIconOption {
        value: "monitor_cog",
        label: "运维",
    },
    NavigationIconOption {
        value: "sliders_horizontal",
        label: "配置",
    },
    NavigationIconOption {
        value: "waypoints",
        label: "节点",
    },
    NavigationIconOption {
        value: "cog",
        label: "系统",
    },
];

#[must_use]
pub fn normalized_navigation_icon(value: &str) -> &'static str {
    NAVIGATION_ICON_OPTIONS
        .iter()
        .find(|option| option.value == value)
        .map_or(DEFAULT_NAVIGATION_ICON, |option| option.value)
}

#[must_use]
pub fn resolved_navigation_icon(configured: Option<&str>, semantic_name: &str) -> &'static str {
    if let Some(configured) = configured
        && let Some(option) = NAVIGATION_ICON_OPTIONS
            .iter()
            .find(|option| option.value == configured)
    {
        return option.value;
    }
    inferred_navigation_icon(semantic_name)
}

#[must_use]
pub fn inferred_navigation_icon(semantic_name: &str) -> &'static str {
    match semantic_name.trim().to_ascii_lowercase().as_str() {
        "home" | "welcome" => "house",
        "users" | "user" => "user_round",
        "roles" | "role" | "permissions" => "shield",
        "departments" | "department" | "organization" => "building_2",
        "dictionary" | "dictionaries" => "book_open",
        "menus" | "menu" => "menu",
        "audit" | "audits" => "clipboard_list",
        "sessions" | "session" | "oauth-clients" => "key_round",
        "tenants" | "tenant" => "landmark",
        "messages" | "message" => "message_square",
        "social-clients" | "social" => "share_2",
        "areas" | "area" => "map_pinned",
        "config" | "settings" => "settings",
        "assets" | "asset" => "package",
        "drive" | "files" => "folder",
        "software" | "modules" => "boxes",
        "device-overview" | "dashboard" => "gauge",
        "algorithms" | "workflow" => "workflow",
        "iot" | "devices" => "cpu",
        "gateway" | "gateways" => "network",
        "ssh" | "terminal" => "terminal",
        "linux" | "servers" => "server",
        "agents" | "agent" => "bot",
        _ => DEFAULT_NAVIGATION_ICON,
    }
}

#[component]
pub fn NavigationIcon(
    name: String,
    #[props(default = "size-4".to_owned())] class: String,
) -> Element {
    match normalized_navigation_icon(&name) {
        "house" => rsx! { House { class } },
        "user_round" => rsx! { UserRound { class } },
        "users_round" => rsx! { UsersRound { class } },
        "shield" => rsx! { Shield { class } },
        "building_2" => rsx! { Building2 { class } },
        "book_open" => rsx! { BookOpen { class } },
        "menu" => rsx! { Menu { class } },
        "clipboard_list" => rsx! { ClipboardList { class } },
        "key_round" => rsx! { KeyRound { class } },
        "landmark" => rsx! { Landmark { class } },
        "message_square" => rsx! { MessageSquare { class } },
        "share_2" => rsx! { Share2 { class } },
        "map_pinned" => rsx! { MapPinned { class } },
        "settings" => rsx! { Settings { class } },
        "package" => rsx! { Package { class } },
        "folder" => rsx! { Folder { class } },
        "database" => rsx! { Database { class } },
        "gauge" => rsx! { Gauge { class } },
        "server" => rsx! { Server { class } },
        "hard_drive" => rsx! { HardDrive { class } },
        "cpu" => rsx! { Cpu { class } },
        "radio" => rsx! { Radio { class } },
        "terminal" => rsx! { Terminal { class } },
        "bot" => rsx! { Bot { class } },
        "cloud" => rsx! { Cloud { class } },
        "workflow" => rsx! { Workflow { class } },
        "network" => rsx! { Network { class } },
        "layers" => rsx! { Layers { class } },
        "boxes" => rsx! { Boxes { class } },
        "file_text" => rsx! { FileText { class } },
        "bell" => rsx! { Bell { class } },
        "contact" => rsx! { Contact { class } },
        "monitor_cog" => rsx! { MonitorCog { class } },
        "sliders_horizontal" => rsx! { SlidersHorizontal { class } },
        "waypoints" => rsx! { Waypoints { class } },
        "cog" => rsx! { Cog { class } },
        "panel_top" => rsx! { PanelTop { class } },
        _ => rsx! { CircleDashed { class } },
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationIconPickerProps {
    pub value: ReadSignal<String>,
    #[props(default)]
    pub name: ReadSignal<String>,
    #[props(default)]
    pub aria_label: ReadSignal<String>,
    #[props(default)]
    pub on_value_change: Callback<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn NavigationIconPicker(props: NavigationIconPickerProps) -> Element {
    let current = normalized_navigation_icon(&(props.value)());
    let attributes = with_class(props.attributes, "dx-navigation-icon-picker".to_owned());

    rsx! {
        fieldset {
            aria_label: props.aria_label,
            ..attributes,
            input {
                r#type: "hidden",
                name: props.name,
                value: current,
            }
            for option in NAVIGATION_ICON_OPTIONS {
                button {
                    key: "{option.value}",
                    class: "dx-navigation-icon-picker__option",
                    r#type: "button",
                    title: option.label,
                    aria_label: option.label,
                    aria_pressed: (current == option.value).to_string(),
                    "data-selected": (current == option.value).to_string(),
                    onclick: move |_| props.on_value_change.call(option.value.to_owned()),
                    NavigationIcon {
                        name: option.value.to_owned(),
                        class: "size-4".to_owned(),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_icons_use_the_default_key() {
        assert_eq!(
            normalized_navigation_icon("unknown"),
            DEFAULT_NAVIGATION_ICON
        );
        assert_eq!(normalized_navigation_icon("settings"), "settings");
    }

    #[test]
    fn explicit_icons_override_inferred_icons() {
        assert_eq!(resolved_navigation_icon(Some("bell"), "users"), "bell");
        assert_eq!(resolved_navigation_icon(Some("用"), "users"), "user_round");
        assert_eq!(resolved_navigation_icon(None, "departments"), "building_2");
    }
}
