#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod application_account;
mod application_fullscreen;
mod application_navigation;
mod application_plugin;
mod application_settings;
mod application_shell;
mod application_shell_model;
mod navigation_validation;
mod page_cache;
mod page_deck;
mod plugin_application;
mod plugin_navigation;
mod url_state;

#[cfg(feature = "workbench")]
mod application_dialog;
#[cfg(feature = "workbench")]
mod extension;
#[cfg(feature = "workbench")]
mod menu_dialog;
#[cfg(feature = "workbench")]
mod navigation;
#[cfg(feature = "workbench")]
mod provider;
#[cfg(feature = "workbench")]
mod runtime;
#[cfg(feature = "workbench")]
mod scene_dialog;
#[cfg(feature = "workbench")]
mod workbench;

pub use application_fullscreen::ApplicationFullscreenPage;
pub use application_plugin::*;
pub use application_settings::{ApplicationSettings, ApplicationSettingsGroup};
pub use application_shell::ApplicationShell;
pub use application_shell_model::*;
#[cfg(feature = "workbench")]
pub use extension::*;
pub use plugin_application::PluginApplication;
#[cfg(feature = "workbench")]
pub use provider::*;
pub use url_state::{PageUrlState, UrlUpdate, use_page_url_state};
#[cfg(feature = "workbench")]
pub use workbench::{AdminShell, App};

#[cfg(feature = "workbench")]
rudi::enable! {}
