use dioxus::prelude::*;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

fn destination(value: &str, base: &str) -> String {
    if value.starts_with('#') {
        return value.into();
    }
    let relative = base.starts_with('/');
    let base_url = if relative {
        format!("https://markdown.invalid{base}")
    } else {
        base.into()
    };
    let Ok(base) = url::Url::parse(&base_url) else {
        return "#".into();
    };
    match base.join(value) {
        Ok(url)
            if matches!(url.scheme(), "http" | "https")
                && url.username().is_empty()
                && url.password().is_none() =>
        {
            if relative && url.host_str() == Some("markdown.invalid") {
                url[url::Position::BeforePath..].to_owned()
            } else {
                url.to_string()
            }
        }
        _ => "#".into(),
    }
}

fn render(source: &str, links: &str, images: &str, references: &[(String, String)]) -> String {
    let mut replacing = false;
    let mut reference_label = None;
    let mut suppress_link = false;
    let events = Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .filter_map(|event| {
        if replacing {
            if matches!(event, Event::End(pulldown_cmark::TagEnd::Link)) {
                replacing = false;
                reference_label = None;
                if suppress_link {
                    suppress_link = false;
                    return None;
                }
                return Some(event);
            }
            return reference_label.take().map(Event::Text);
        }
        match event {
            Event::Html(_) | Event::InlineHtml(_) => None,
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                if let Some((index, (_, label))) = references
                    .iter()
                    .enumerate()
                    .find(|(_, (uri, _))| uri == dest_url.as_ref())
                {
                    replacing = true;
                    reference_label = Some(CowStr::from(label.clone()));
                    Some(Event::Start(Tag::Link {
                        link_type,
                        dest_url: CowStr::from(format!("#aio-reference-{index}")),
                        title,
                        id,
                    }))
                } else {
                    let href = destination(&dest_url, links);
                    if href == "#" || href.starts_with("#aio-reference-") {
                        replacing = true;
                        suppress_link = true;
                        Some(Event::Text(CowStr::from("来源不可用")))
                    } else {
                        Some(Event::Start(Tag::Link {
                            link_type,
                            dest_url: CowStr::from(href),
                            title,
                            id,
                        }))
                    }
                }
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => Some(Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::from(destination(&dest_url, images)),
                title,
                id,
            })),
            event => Some(event),
        }
    });
    let mut rendered = String::new();
    html::push_html(&mut rendered, events);
    rendered
}

#[component]
pub fn Markdown(
    source: String,
    link_base: String,
    image_base: String,
    #[props(default)] references: Vec<(String, String)>,
    #[props(default)] on_reference: Callback<String>,
) -> Element {
    let id = use_signal(|| {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        format!(
            "markdown-{}",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    });
    let mut listener = use_signal(|| None::<document::Eval>);
    use_drop(move || {
        if let Some(listener) = listener() {
            let _ = listener.send(());
        }
    });
    let content = render(&source, &link_base, &image_base, &references);
    let available = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(references.clone())));
    *available.borrow_mut() = references;
    rsx! {
        article {
            id: id(), class: "dx-markdown", dangerous_inner_html: content,
            onmounted: move |_| {
                let mut events = document::eval(r##"
                    const id = await dioxus.recv();
                    const root = document.getElementById(id);
                    const handle = event => {
                        const link = event.target.closest('a[href^="#aio-reference-"]');
                        if (!link || !root.contains(link)) return;
                        event.preventDefault();
                        dioxus.send(Number(link.getAttribute('href').slice('#aio-reference-'.length)));
                    };
                    root.addEventListener('click', handle);
                    await dioxus.recv(); root.removeEventListener('click', handle);
                "##);
                if events.send(id()).is_err() { return; }
                listener.set(Some(events));
                let available = available.clone();
                spawn(async move {
                    while let Ok(index) = events.recv::<usize>().await {
                        if let Some((uri, _)) = available.borrow().get(index) { on_reference.call(uri.clone()); }
                    }
                });
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_require_explicit_authorization_and_use_trusted_labels() {
        let html = render(
            "[伪造标题](memory:allowed) [不可信](memory:unknown) [假入口](#aio-reference-0)",
            "https://app.test/",
            "https://app.test/",
            &[("memory:allowed".into(), "可信标题".into())],
        );
        assert!(html.contains("href=\"#aio-reference-0\">可信标题</a>"));
        assert!(!html.contains("伪造标题"));
        assert!(!html.contains("memory:unknown"));
        assert_eq!(html.matches("<a ").count(), 1);
    }

    #[test]
    fn strips_executable_content_and_resolves_versioned_assets() {
        let html = render(
            "# Heading\n\n<script>alert(1)</script>\n\n[x](javascript:alert(1))\n\n![preview](docs/screen.png)\n\n| a | b |\n|---|---|\n| 1 | 2 |",
            "https://github.com/example/repo/blob/abc/",
            "https://aio.test/api/docs/abc/",
            &[],
        );
        assert!(!html.contains("<script"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("https://aio.test/api/docs/abc/docs/screen.png"));
        assert!(html.contains("<table>"));
    }
}
