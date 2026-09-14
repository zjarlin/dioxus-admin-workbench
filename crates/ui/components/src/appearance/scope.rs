use az_admin_shell_core::appearance::AppearancePreferences;
use dioxus::prelude::*;

#[derive(Clone)]
pub(super) struct AppearanceState {
    pub preferences: Signal<AppearancePreferences>,
    pub loaded: Signal<bool>,
    pub feedback: Signal<Option<(bool, String)>>,
    storage_key: String,
}

#[component]
pub fn AppearanceScope(user_key: String, children: Element) -> Element {
    let preferences = use_signal(AppearancePreferences::default);
    let loaded = use_signal(|| false);
    let feedback = use_signal(|| None);
    let state = use_context_provider(|| AppearanceState {
        preferences,
        loaded,
        feedback,
        storage_key: format!(
            "workbench:appearance:v1:{}",
            serde_json::to_string(&user_key).unwrap()
        ),
    });
    use_future(move || {
        let mut state = state.clone();
        async move {
            let key = serde_json::to_string(&state.storage_key).unwrap();
            let result = document::eval(&format!(
                "try {{ return localStorage.getItem({key}); }} catch (_) {{ return null; }}"
            ))
            .await;
            let value = result
                .ok()
                .and_then(|v| v.as_str().and_then(|s| serde_json::from_str(s).ok()))
                .unwrap_or_default();
            state.preferences.set(value);
            apply(value).await;
            state.loaded.set(true);
        }
    });
    rsx! { {children} }
}

pub(super) async fn save(mut state: AppearanceState, value: AppearancePreferences) {
    state.preferences.set(value);
    apply(value).await;
    let key = serde_json::to_string(&state.storage_key).unwrap();
    let payload = serde_json::to_string(&serde_json::to_string(&value).unwrap()).unwrap();
    let saved = document::eval(&format!("try {{ localStorage.setItem({key},{payload}); return true; }} catch (_) {{ return false; }}")).await;
    state
        .feedback
        .set(Some(if saved.is_ok_and(|v| v.as_bool() == Some(true)) {
            (false, "外观已保存到当前设备".into())
        } else {
            (
                true,
                "外观已应用，但浏览器未允许保存；重新打开后将恢复默认".into(),
            )
        }));
}

async fn apply(value: AppearancePreferences) {
    let json = serde_json::to_string(&value).unwrap();
    let _ = document::eval(&format!(
        "const preference={json};{}",
        include_str!("apply.js")
    ))
    .await;
}
