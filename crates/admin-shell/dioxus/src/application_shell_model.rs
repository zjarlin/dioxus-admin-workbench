#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub page_id: Option<String>,
    pub enabled: bool,
    pub children: Vec<ApplicationMenuItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSceneItem {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationUser {
    pub label: String,
    pub handle: String,
    pub initials: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationAccountItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub page_id: Option<String>,
    pub destructive: bool,
}

/// 运行时插件贡献的可序列化页面入口。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRuntimePage {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub scene_id: String,
    pub scene_label: String,
    pub definition: String,
}
