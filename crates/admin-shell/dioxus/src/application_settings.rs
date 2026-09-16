use dioxus::prelude::*;

/// 宿主已授权的设置分组，标题来自插件声明，不参与插件身份解析。
#[derive(Clone, Debug, PartialEq)]
pub struct ApplicationSettingsGroup {
    pub page_id: String,
    pub title: String,
}

/// 宿主提供给设置中心的扩展槽；页面挂载与授权仍由宿主执行。
#[derive(Clone, Copy)]
pub struct ApplicationSettings {
    pub selected: Signal<Option<String>>,
    pub groups: ReadSignal<Vec<ApplicationSettingsGroup>>,
    pub render: Callback<String, Element>,
}
