use crate::{
    ApplicationFullscreenPage, ApplicationRuntimePage,
    page_cache::{PageCache, PageSource},
};
use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[component]
pub(super) fn PageDeck(
    pages: Vec<PageSource>,
    selected: Option<String>,
    active: bool,
    capacity: usize,
    renderer: Option<Callback<ApplicationRuntimePage, Element>>,
    #[props(default)] fullscreen: Option<String>,
    #[props(default)] on_back: Option<Callback<()>>,
) -> Element {
    let cache = use_hook(|| Rc::new(RefCell::new(PageCache::default())));
    let entries = cache
        .borrow_mut()
        .reconcile(&pages, selected.as_deref(), capacity);
    rsx! {
        for entry in entries {
            div {
                key: "{entry.serial}",
                hidden: selected.as_deref() != Some(entry.source.id()),
                "data-aio-page": entry.source.id(),
                "data-aio-page-active": (active && selected.as_deref() == Some(entry.source.id())).to_string(),
                if let Some(label) = fullscreen.as_ref() {
                    ApplicationFullscreenPage {
                        application_label: label.clone(),
                        page_label: entry.source.label().to_owned(),
                        on_back: move |_| { if let Some(callback) = on_back { callback.call(()); } },
                        CachedPage { source: entry.source.clone(), renderer }
                    }
                } else {
                    CachedPage { source: entry.source.clone(), renderer }
                }
            }
        }
    }
}

#[component]
fn CachedPage(
    source: PageSource,
    renderer: Option<Callback<ApplicationRuntimePage, Element>>,
) -> Element {
    match source {
        PageSource::Native(page) => (page.render)(),
        PageSource::Runtime(page, _) => renderer
            .map(|render| render.call(page))
            .unwrap_or_else(|| rsx! {}),
    }
}
