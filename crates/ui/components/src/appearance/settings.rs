use super::scope::{AppearanceState, save};
use crate::{
    admin::StatusMessage,
    button::{Button, ButtonVariant},
};
use az_admin_shell_core::appearance::{Density, Theme};
use dioxus::prelude::*;

#[component]
pub fn AppearanceSettings() -> Element {
    let state = use_context::<AppearanceState>();
    let value = (state.preferences)();
    rsx! {
        div { class: "workbench-preferences",
            fieldset { class: "workbench-preferences__field", disabled: !(state.loaded)(),
                legend { "主题" }
                div { class: "workbench-preferences__choices",
                    for (theme, label) in [(Theme::System, "跟随系统"), (Theme::Light, "浅色"), (Theme::Dark, "深色")] {
                        Button { variant: ButtonVariant::Outline, aria_pressed: (value.theme == theme).to_string(),
                            onclick: { let state = state.clone(); move |_| { let state = state.clone(); spawn(async move { save(state, az_admin_shell_core::appearance::AppearancePreferences { theme, ..value }).await; }); } }, "{label}" }
                    }
                }
            }
            fieldset { class: "workbench-preferences__field", disabled: !(state.loaded)(),
                legend { "显示密度" }
                div { class: "workbench-preferences__choices",
                    for (density, label) in [(Density::Comfortable, "舒适"), (Density::Compact, "紧凑")] {
                        Button { variant: ButtonVariant::Outline, aria_pressed: (value.density == density).to_string(),
                            onclick: { let state = state.clone(); move |_| { let state = state.clone(); spawn(async move { save(state, az_admin_shell_core::appearance::AppearancePreferences { density, ..value }).await; }); } }, "{label}" }
                    }
                }
            }
            p { class: "admin-meta", "仅在当前设备为此账户保存，切换工作区仍然生效。" }
            if let Some((error, message)) = (state.feedback)() { StatusMessage { error, message } }
        }
    }
}
