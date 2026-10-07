use std::collections::BTreeMap;

use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Default, Deserialize, PartialEq)]
pub(crate) struct UrlLocation {
    pub page: Option<String>,
    pub account: Option<String>,
    pub parameters: BTreeMap<String, String>,
}

impl UrlLocation {
    fn active_page(&self) -> Option<&str> {
        self.account.as_deref().or(self.page.as_deref())
    }
}

/// 已提交的导航创建历史节点，连续输入合并为一个节点。
#[derive(Clone, Copy)]
pub enum UrlUpdate {
    Push,
    Replace,
    Continuous,
}

impl UrlUpdate {
    fn mode(self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Replace => "replace",
            Self::Continuous => "continuous",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UrlNavigation {
    pub location: Signal<Option<UrlLocation>>,
}

impl UrlNavigation {
    pub fn navigate(self, page: Option<&str>, account: Option<&str>, mode: UrlUpdate) {
        command(
            "navigate",
            json!({ "page": page, "account": account, "mode": mode.mode() }),
        );
    }

    pub fn scope(self, scope: &str, default_page: Option<&str>) {
        command("setScope", json!({ "scope": scope, "page": default_page }));
    }
}

pub(crate) fn use_url_navigation() -> UrlNavigation {
    let mut location = use_signal(|| None::<UrlLocation>);
    let mut bridge = use_signal(|| None::<document::Eval>);
    use_drop(move || {
        if let Some(evaluation) = bridge() {
            let _ = evaluation.send(json!({ "dispose": true }));
        }
    });
    use_future(move || async move {
        let script = format!(
            "{}\nconst navigation = window.__adminUrlState ||= createApplicationUrlState(window);\n\
             const unsubscribe = navigation.subscribe(value => dioxus.send(value));\n\
             try {{ await dioxus.recv(); }} finally {{ unsubscribe(); }}",
            include_str!("url_state.js")
        );
        let mut evaluation = document::eval(&script);
        bridge.set(Some(evaluation));
        while let Ok(value) = evaluation.recv::<UrlLocation>().await {
            location.set(Some(value));
        }
    });
    use_context_provider(|| UrlNavigation { location })
}

/// 原生页面读取自己的 URL 参数；后台保活页面不会被其他页面的参数覆盖。
#[derive(Clone, Copy)]
pub struct PageUrlState {
    page_id: &'static str,
    parameters: Signal<BTreeMap<String, String>>,
}

impl PageUrlState {
    pub fn value(self, name: &str) -> Option<String> {
        self.parameters.read().get(&format!("view.{name}")).cloned()
    }

    /// 一次更新同时提交多个相关字段，避免中间状态成为历史节点。
    pub fn update(self, values: &[(&str, Option<String>)], mode: UrlUpdate) {
        let values = values
            .iter()
            .map(|(name, value)| (format!("view.{name}"), value))
            .collect::<BTreeMap<_, _>>();
        command(
            "update",
            json!({ "page": self.page_id, "values": values, "mode": mode.mode() }),
        );
    }
}

/// 由 PluginApplication 提供上下文，页面使用稳定业务 ID 声明参数归属。
pub fn use_page_url_state(page_id: &'static str) -> PageUrlState {
    let navigation = use_context::<UrlNavigation>();
    let initial = navigation
        .location
        .peek()
        .as_ref()
        .filter(|value| value.active_page() == Some(page_id))
        .map(|value| value.parameters.clone())
        .unwrap_or_default();
    let mut parameters = use_signal(|| initial);
    use_effect(move || {
        let location = navigation.location.read();
        if let Some(value) = location
            .as_ref()
            .filter(|value| value.active_page() == Some(page_id))
        {
            if *parameters.peek() != value.parameters {
                parameters.set(value.parameters.clone());
            }
        }
    });
    PageUrlState {
        page_id,
        parameters,
    }
}

fn command(method: &str, payload: Value) {
    let script = format!("window.__adminUrlState?.{method}({payload});");
    spawn(async move {
        if let Err(error) = document::eval(&script).await {
            tracing::error!("更新页面 URL 失败: {error}");
        }
    });
}
