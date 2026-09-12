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

fn render(source: &str, links: &str, images: &str) -> String {
    let events = Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .filter_map(|event| match event {
        Event::Html(_) | Event::InlineHtml(_) => None,
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Some(Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::from(destination(&dest_url, links)),
            title,
            id,
        })),
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
    });
    let mut rendered = String::new();
    html::push_html(&mut rendered, events);
    rendered
}

#[component]
pub fn Markdown(source: String, link_base: String, image_base: String) -> Element {
    let content = render(&source, &link_base, &image_base);
    rsx! { article { class: "dx-markdown", dangerous_inner_html: content } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_executable_content_and_resolves_versioned_assets() {
        let html = render(
            "# Heading\n\n<script>alert(1)</script>\n\n[x](javascript:alert(1))\n\n![preview](docs/screen.png)\n\n| a | b |\n|---|---|\n| 1 | 2 |",
            "https://github.com/example/repo/blob/abc/",
            "https://aio.test/api/docs/abc/",
        );
        assert!(!html.contains("<script"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("https://aio.test/api/docs/abc/docs/screen.png"));
        assert!(html.contains("<table>"));
    }
}
