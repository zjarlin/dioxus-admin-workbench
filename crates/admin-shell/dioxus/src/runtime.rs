use az_admin_shell_core::{CompiledPage, CompiledPageRenderer, ResourceCatalog};
use dioxus::prelude::*;

use crate::{
    AdminProviderHandle, ConventionPageContext, ConventionPageIndex, PageExtensionRendererIndex,
    PageExtensionRuntimeContext,
};

pub(crate) fn render_page(
    page: CompiledPage,
    resources: ResourceCatalog,
    admin: AdminProviderHandle,
    extensions: &PageExtensionRendererIndex,
    convention_pages: &ConventionPageIndex,
) -> Element {
    match page.renderer.clone() {
        CompiledPageRenderer::ConventionFile {
            module_name,
            expected_path,
        } => {
            let Some(provider) = convention_pages.get(&module_name) else {
                return error_state(
                    "约定页面尚未进入构建",
                    &format!("期望文件: {expected_path}"),
                );
            };
            provider.render(ConventionPageContext { page, admin })
        }
        CompiledPageRenderer::Extension {
            extension_type,
            provider_key,
            payload,
            ..
        } => {
            let Some(provider) = extensions.get(&extension_type) else {
                return error_state("页面扩展未注册", &provider_key);
            };
            if provider.key() != provider_key {
                return error_state(
                    "页面扩展身份不一致",
                    &format!("{} != {provider_key}", provider.key()),
                );
            }
            provider.render(PageExtensionRuntimeContext {
                page,
                payload,
                resources,
                admin,
            })
        }
    }
}

pub(crate) fn error_state(title: &str, message: &str) -> Element {
    rsx! {
        section { role: "alert",
            h2 { "{title}" }
            p { "{message}" }
        }
    }
}
