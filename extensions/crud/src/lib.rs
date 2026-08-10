#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod compiler;
mod config;
mod editor;
mod runtime;

use std::sync::Arc;

use az_admin_shell_core::DynPageExtensionCompiler;
use az_dioxus_admin_shell::{
    DynPageExtensionRenderer, PageExtensionEditorContext, PageExtensionRenderer,
    PageExtensionRuntimeContext,
};
use dioxus::prelude::*;

pub use config::{CrudPageConfig, CrudPageExtension};

impl PageExtensionRenderer for CrudPageExtension {
    fn key(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn render_editor(&self, context: PageExtensionEditorContext) -> Element {
        rsx! { editor::CrudEditor { context } }
    }

    fn render(&self, context: PageExtensionRuntimeContext) -> Element {
        runtime::render(context)
    }
}

#[rudi::Singleton(name = std::any::type_name::<CrudPageExtension>())]
fn crud_compiler() -> DynPageExtensionCompiler {
    Arc::new(CrudPageExtension)
}

#[rudi::Singleton(name = std::any::type_name::<CrudPageExtension>())]
fn crud_renderer() -> DynPageExtensionRenderer {
    Arc::new(CrudPageExtension)
}

rudi::enable! {}
