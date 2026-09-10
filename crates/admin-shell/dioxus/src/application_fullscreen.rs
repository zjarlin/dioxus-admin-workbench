use az_ui_components::{
    UiStylesheets,
    button::{Button, ButtonSize, ButtonVariant},
};
use dioxus::prelude::*;
use icons::ArrowLeft;

#[component]
pub fn ApplicationFullscreenPage(
    application_label: String,
    page_label: String,
    on_back: Callback<()>,
    children: Element,
) -> Element {
    rsx! {
        UiStylesheets {}
        section { class: "application-fullscreen", aria_label: "{application_label}",
            header { class: "application-fullscreen__header",
                Button {
                    class: "application-fullscreen__back",
                    r#type: "button",
                    size: ButtonSize::Sm,
                    variant: ButtonVariant::Ghost,
                    title: "返回主后台",
                    aria_label: "返回主后台",
                    onclick: move |_| on_back.call(()),
                    ArrowLeft { class: "size-4" }
                    span { "返回主后台" }
                }
                h1 { "{page_label}" }
            }
            main { class: "application-fullscreen__content", aria_label: "{page_label}",
                {children}
            }
        }
    }
}
