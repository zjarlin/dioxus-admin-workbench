#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod application_dialog;
mod extension;
mod menu_dialog;
mod navigation;
mod provider;
mod runtime;
mod scene_dialog;
mod workbench;

pub use extension::*;
pub use provider::*;
pub use workbench::{AdminShell, App};

rudi::enable! {}
