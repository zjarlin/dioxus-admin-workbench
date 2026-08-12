use crate::{
    attributes::with_class,
    button::{Button, ButtonSize},
    textarea::Textarea,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bot, LoaderCircle, Send, UserRound};

const AGENT_CHAT_CLASS: &str = "dx-agent-chat";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentChatRole {
    User,
    Agent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentChatMessage {
    pub id: String,
    pub role: AgentChatRole,
    pub content: String,
}

impl AgentChatMessage {
    pub fn user(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: AgentChatRole::User,
            content: content.into(),
        }
    }

    pub fn agent(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: AgentChatRole::Agent,
            content: content.into(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AgentChatProps {
    pub aria_label: String,
    pub messages: Vec<AgentChatMessage>,
    pub value: ReadSignal<String>,
    #[props(default)]
    pub busy: ReadSignal<bool>,
    #[props(default = "描述需要修改的内容".to_owned())]
    pub placeholder: String,
    #[props(default = "直接描述修改要求".to_owned())]
    pub empty_text: String,
    #[props(default)]
    pub on_value_change: Callback<String>,
    #[props(default)]
    pub on_submit: Callback<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn AgentChat(props: AgentChatProps) -> Element {
    let attributes = with_class(props.attributes, AGENT_CHAT_CLASS.to_owned());
    let submit_value = props.value;
    let submit_busy = props.busy;
    let on_submit = props.on_submit;
    let submit = use_callback(move |_: ()| {
        let prompt = submit_value().trim().to_owned();
        if prompt.is_empty() || submit_busy() {
            return;
        }
        on_submit.call(prompt);
    });

    rsx! {
        section { aria_label: props.aria_label, ..attributes,
            div { class: "dx-agent-chat__messages", aria_live: "polite",
                if props.messages.is_empty() {
                    div { class: "dx-agent-chat__empty",
                        Bot { class: "size-4" }
                        span { "{props.empty_text}" }
                    }
                }
                for message in &props.messages {
                    article {
                        key: "{message.id}",
                        class: if message.role == AgentChatRole::User {
                            "dx-agent-chat__message dx-agent-chat__message--user"
                        } else {
                            "dx-agent-chat__message dx-agent-chat__message--agent"
                        },
                        if message.role == AgentChatRole::User {
                            UserRound { class: "size-4" }
                        } else {
                            Bot { class: "size-4" }
                        }
                        p { "{message.content}" }
                    }
                }
                if (props.busy)() {
                    div { class: "dx-agent-chat__pending", role: "status",
                        LoaderCircle { class: "size-4" }
                        span { "Agent 正在修改" }
                    }
                }
            }
            form {
                class: "dx-agent-chat__composer",
                onsubmit: move |event| {
                    event.prevent_default();
                    submit.call(());
                },
                Textarea {
                    aria_label: "Agent 修改要求",
                    rows: "2",
                    disabled: props.busy,
                    placeholder: props.placeholder,
                    value: props.value,
                    oninput: move |event: FormEvent| props.on_value_change.call(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Enter && !event.modifiers().shift() {
                            event.prevent_default();
                            submit.call(());
                        }
                    },
                }
                Button {
                    r#type: "submit",
                    size: ButtonSize::Icon,
                    disabled: (props.busy)() || (props.value)().trim().is_empty(),
                    title: "发送修改要求",
                    aria_label: "发送修改要求",
                    Send { class: "size-4" }
                }
            }
        }
    }
}
